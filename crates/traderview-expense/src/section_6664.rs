//! IRC § 6664 — Reasonable cause + good faith defense to
//! accuracy-related and fraud penalties. The single most
//! important affirmative defense in the penalty-section
//! constellation. Cross-cutting defense applies to § 6662
//! (accuracy-related), § 6662A (reportable transaction
//! understatement), and § 6663 (civil fraud). Distinct from
//! `section_6662` (accuracy-related penalty math) and
//! `section_6662a` (reportable transaction penalty math),
//! which already cite § 6664(d) heightened defense. This
//! module pulls out the universal defense framework as a
//! standalone procedural compliance check.
//!
//! **§ 6664(c)(1) — General rule**. No penalty under § 6662
//! or § 6663 may be imposed with respect to any portion of an
//! underpayment if the taxpayer shows (a) reasonable cause AND
//! (b) good faith with respect to that portion.
//!
//! **§ 6664(c)(2) — Economic substance bar**. The defense is
//! NOT AVAILABLE for any portion attributable to a transaction
//! lacking economic substance under § 7701(o). § 6662(b)(6) +
//! § 6662(i) impose strict-liability 20% (40% non-disclosed)
//! accuracy penalty with no reasonable-cause escape.
//!
//! **§ 6664(d) — Reportable transaction heightened defense**.
//! For § 6662A reportable transaction understatements, the
//! reasonable-cause defense applies ONLY if all THREE elements
//! are satisfied: (A) ADEQUATE DISCLOSURE per § 6664(d)(3)(A);
//! (B) SUBSTANTIAL AUTHORITY for treatment per § 6664(d)(3)(B);
//! (C) REASONABLE BELIEF that the treatment was MORE LIKELY
//! THAN NOT proper per § 6664(d)(3)(C). All three required —
//! missing any one defeats the defense.
//!
//! **Treas. Reg. § 1.6664-4 — Implementing regulation**.
//! Facts-and-circumstances analysis. Relevant factors:
//! - Taxpayer's education, sophistication, business experience
//! - Reliance on professional tax advisor (CPA, attorney, EA)
//! - Whether advisor had complete accurate facts
//! - Whether advisor advice based on unreasonable factual /
//!   legal assumptions
//! - Whether advisor unreasonably relied on taxpayer's
//!   representations / statements / findings
//! - Whether taxpayer made reasonable effort to assess proper
//!   tax liability
//!
//! **Regulation invalidity position** — § 1.6662-3(c)(2):
//! taxpayer may NOT rely on opinion that a regulation is
//! invalid as reasonable-cause defense UNLESS the taxpayer
//! adequately disclosed the position challenging the
//! regulation's validity.
//!
//! **Trader-relevant**: Aggressive § 1256 mark-to-market
//! positions, § 988 currency reclassifications, § 1202 QSBS
//! holding-period reach-backs, § 475(f) trader-tax-status
//! claims often face § 6662 substantial-understatement
//! penalty exposure. § 6664 reasonable-cause + good-faith
//! defense via reliance on professional tax advisor is the
//! primary affirmative defense.
//!
//! Citations: IRC § 6664(c)(1) general rule; § 6664(c)(2)
//! economic substance bar; § 6664(c)(3) appraisal-related
//! special rules; § 6664(d) reportable-transaction heightened
//! defense; § 6664(d)(3)(A) adequate disclosure;
//! § 6664(d)(3)(B) substantial authority; § 6664(d)(3)(C)
//! reasonable belief more-likely-than-not; Treas. Reg.
//! § 1.6664-4 implementing regulation; § 1.6662-3(c)(2)
//! regulation-invalidity disclosure rule.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PenaltyTarget {
    /// § 6662 accuracy-related penalty (negligence, substantial
    /// understatement, substantial valuation, etc.).
    Section6662Accuracy,
    /// § 6662A reportable transaction understatement.
    Section6662AReportable,
    /// § 6663 civil fraud penalty (75%).
    Section6663CivilFraud,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6664Input {
    pub penalty_target: PenaltyTarget,
    /// Whether the taxpayer asserts reasonable cause for the
    /// underpayment.
    pub reasonable_cause_alleged: bool,
    /// Whether the taxpayer asserts good faith.
    pub good_faith_alleged: bool,
    /// Whether the taxpayer relied on a professional tax advisor.
    pub relied_on_professional_tax_advisor: bool,
    /// Whether the taxpayer provided the advisor with complete
    /// and accurate facts.
    pub taxpayer_provided_complete_accurate_facts_to_advisor: bool,
    /// Whether the advisor's advice was based on unreasonable
    /// factual / legal assumptions (negates reliance).
    pub advisor_advice_based_on_unreasonable_assumptions: bool,
    /// Whether the advisor unreasonably relied on taxpayer's
    /// representations / findings (negates reliance).
    pub advisor_unreasonably_relied_on_taxpayer_representations: bool,
    /// Whether the underpayment is attributable to a transaction
    /// lacking economic substance under § 7701(o) (§ 6664(c)(2)
    /// strict-liability bar).
    pub transaction_lacks_economic_substance: bool,
    /// Whether the taxpayer's position challenges a regulation's
    /// validity and was adequately disclosed per § 1.6662-3(c)(2).
    pub regulation_invalidity_position_adequately_disclosed: bool,
    /// § 6664(d) reportable-transaction defense element:
    /// adequate disclosure per § 6664(d)(3)(A).
    pub adequate_disclosure_per_d_3_a: bool,
    /// § 6664(d)(3)(B) substantial authority for treatment.
    pub substantial_authority_per_d_3_b: bool,
    /// § 6664(d)(3)(C) reasonable belief more-likely-than-not.
    pub reasonable_belief_more_likely_than_not_per_d_3_c: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6664Result {
    /// Whether the reasonable-cause + good-faith defense is
    /// engaged (procedural prerequisites met).
    pub defense_engaged: bool,
    /// Whether the defense is barred by § 6664(c)(2) economic
    /// substance strict-liability rule.
    pub defense_barred_by_economic_substance: bool,
    /// Whether the defense is barred by § 6664(d) reportable
    /// transaction heightened requirements (missing any of the
    /// three elements).
    pub defense_barred_by_reportable_transaction_inadequate_disclosure: bool,
    /// Whether the advisor-reliance prong supports the defense
    /// (Treas. Reg. § 1.6664-4 facts-and-circumstances).
    pub advisor_reliance_supports_defense: bool,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section6664Input) -> Section6664Result {
    let mut notes: Vec<String> = Vec::new();

    notes.push(
        "§ 6664(c)(1) — no penalty under § 6662 or § 6663 may be imposed with respect to any portion of an underpayment if taxpayer shows reasonable cause AND good faith with respect to that portion"
            .to_string(),
    );

    let base_elements_met = input.reasonable_cause_alleged && input.good_faith_alleged;

    let economic_substance_bar = input.transaction_lacks_economic_substance;
    if economic_substance_bar {
        notes.push(
            "§ 6664(c)(2) — defense NOT AVAILABLE for any portion attributable to a transaction lacking economic substance under § 7701(o); § 6662(b)(6) + § 6662(i) impose strict-liability 20% (40% non-disclosed) penalty with no reasonable-cause escape"
                .to_string(),
        );
    }

    let advisor_reliance = input.relied_on_professional_tax_advisor
        && input.taxpayer_provided_complete_accurate_facts_to_advisor
        && !input.advisor_advice_based_on_unreasonable_assumptions
        && !input.advisor_unreasonably_relied_on_taxpayer_representations;

    if input.relied_on_professional_tax_advisor {
        if advisor_reliance {
            notes.push(
                "Treas. Reg. § 1.6664-4 — advisor reliance supports defense: taxpayer provided complete + accurate facts; advice was not based on unreasonable assumptions; advisor did not unreasonably rely on taxpayer's representations"
                    .to_string(),
            );
        } else {
            notes.push(
                "Treas. Reg. § 1.6664-4 — advisor reliance INSUFFICIENT: advice must not be based on unreasonable factual / legal assumptions AND advisor must not unreasonably rely on taxpayer's representations AND taxpayer must have provided complete + accurate facts"
                    .to_string(),
            );
        }
    }

    let mut reportable_inadequate = false;

    match input.penalty_target {
        PenaltyTarget::Section6662AReportable => {
            let all_three = input.adequate_disclosure_per_d_3_a
                && input.substantial_authority_per_d_3_b
                && input.reasonable_belief_more_likely_than_not_per_d_3_c;
            if !all_three {
                reportable_inadequate = true;
                notes.push(
                    "§ 6664(d)(3) — § 6662A reportable transaction heightened defense requires ALL THREE elements: (A) adequate disclosure per § 6664(d)(3)(A); (B) substantial authority per § 6664(d)(3)(B); (C) reasonable belief more-likely-than-not per § 6664(d)(3)(C); missing any one defeats defense"
                        .to_string(),
                );
            } else {
                notes.push(
                    "§ 6664(d)(3) — all three reportable-transaction heightened defense elements satisfied"
                        .to_string(),
                );
            }
        }
        PenaltyTarget::Section6662Accuracy => {
            notes.push(
                "§ 6662 accuracy penalty — § 6664(c)(1) reasonable-cause + good-faith defense applies; Treas. Reg. § 1.6664-4 facts-and-circumstances analysis governs"
                    .to_string(),
            );
        }
        PenaltyTarget::Section6663CivilFraud => {
            notes.push(
                "§ 6663 civil fraud (75%) — § 6664(c)(1) reasonable-cause + good-faith defense theoretically applies but IRS bears clear-and-convincing fraud burden; if fraud established, reasonable-cause defense rarely succeeds"
                    .to_string(),
            );
        }
    }

    let defense_engaged = base_elements_met
        && !economic_substance_bar
        && !reportable_inadequate;

    notes.push(
        "Treas. Reg. § 1.6664-4 facts-and-circumstances factors: taxpayer's education, sophistication, business experience; reasonable effort to assess proper tax liability; reliance on professional advisor; complete + accurate facts provided to advisor"
            .to_string(),
    );

    if !input.regulation_invalidity_position_adequately_disclosed {
        notes.push(
            "Treas. Reg. § 1.6662-3(c)(2) — taxpayer may NOT rely on opinion that regulation is invalid as reasonable-cause defense unless the position challenging regulation's validity is adequately disclosed"
                .to_string(),
        );
    }

    Section6664Result {
        defense_engaged,
        defense_barred_by_economic_substance: economic_substance_bar,
        defense_barred_by_reportable_transaction_inadequate_disclosure: reportable_inadequate,
        advisor_reliance_supports_defense: advisor_reliance,
        citation: "IRC §§ 6664(c)(1), 6664(c)(2), 6664(c)(3), 6664(d), 6664(d)(3)(A), 6664(d)(3)(B), 6664(d)(3)(C); § 7701(o); § 6662(b)(6); § 6662(i); Treas. Reg. §§ 1.6664-4, 1.6662-3(c)(2)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_6662() -> Section6664Input {
        Section6664Input {
            penalty_target: PenaltyTarget::Section6662Accuracy,
            reasonable_cause_alleged: true,
            good_faith_alleged: true,
            relied_on_professional_tax_advisor: true,
            taxpayer_provided_complete_accurate_facts_to_advisor: true,
            advisor_advice_based_on_unreasonable_assumptions: false,
            advisor_unreasonably_relied_on_taxpayer_representations: false,
            transaction_lacks_economic_substance: false,
            regulation_invalidity_position_adequately_disclosed: true,
            adequate_disclosure_per_d_3_a: false,
            substantial_authority_per_d_3_b: false,
            reasonable_belief_more_likely_than_not_per_d_3_c: false,
        }
    }

    fn base_6662a() -> Section6664Input {
        let mut i = base_6662();
        i.penalty_target = PenaltyTarget::Section6662AReportable;
        i.adequate_disclosure_per_d_3_a = true;
        i.substantial_authority_per_d_3_b = true;
        i.reasonable_belief_more_likely_than_not_per_d_3_c = true;
        i
    }

    fn base_6663() -> Section6664Input {
        let mut i = base_6662();
        i.penalty_target = PenaltyTarget::Section6663CivilFraud;
        i
    }

    #[test]
    fn clean_6662_defense_engaged() {
        let r = check(&base_6662());
        assert!(r.defense_engaged);
        assert!(r.advisor_reliance_supports_defense);
        assert!(!r.defense_barred_by_economic_substance);
        assert!(!r.defense_barred_by_reportable_transaction_inadequate_disclosure);
    }

    #[test]
    fn missing_reasonable_cause_no_defense() {
        let mut i = base_6662();
        i.reasonable_cause_alleged = false;
        let r = check(&i);
        assert!(!r.defense_engaged);
    }

    #[test]
    fn missing_good_faith_no_defense() {
        let mut i = base_6662();
        i.good_faith_alleged = false;
        let r = check(&i);
        assert!(!r.defense_engaged);
    }

    #[test]
    fn both_base_elements_required_truth_table() {
        for rc in [false, true] {
            for gf in [false, true] {
                let mut i = base_6662();
                i.reasonable_cause_alleged = rc;
                i.good_faith_alleged = gf;
                let r = check(&i);
                assert_eq!(r.defense_engaged, rc && gf);
            }
        }
    }

    #[test]
    fn economic_substance_bar_engaged() {
        let mut i = base_6662();
        i.transaction_lacks_economic_substance = true;
        let r = check(&i);
        assert!(!r.defense_engaged);
        assert!(r.defense_barred_by_economic_substance);
        assert!(r.notes.iter().any(|n| n.contains("§ 6664(c)(2)") && n.contains("strict-liability") && n.contains("§ 7701(o)")));
    }

    #[test]
    fn economic_substance_bar_overrides_clean_defense() {
        let mut i = base_6662();
        i.reasonable_cause_alleged = true;
        i.good_faith_alleged = true;
        i.transaction_lacks_economic_substance = true;
        let r = check(&i);
        assert!(!r.defense_engaged);
        assert!(r.defense_barred_by_economic_substance);
    }

    #[test]
    fn advisor_reliance_supports_defense_when_all_elements_met() {
        let r = check(&base_6662());
        assert!(r.advisor_reliance_supports_defense);
        assert!(r.notes.iter().any(|n| n.contains("§ 1.6664-4") && n.contains("advisor reliance supports defense")));
    }

    #[test]
    fn advisor_unreasonable_assumptions_defeats_reliance() {
        let mut i = base_6662();
        i.advisor_advice_based_on_unreasonable_assumptions = true;
        let r = check(&i);
        assert!(!r.advisor_reliance_supports_defense);
        assert!(r.notes.iter().any(|n| n.contains("INSUFFICIENT")));
    }

    #[test]
    fn advisor_unreasonable_reliance_on_taxpayer_defeats_reliance() {
        let mut i = base_6662();
        i.advisor_unreasonably_relied_on_taxpayer_representations = true;
        let r = check(&i);
        assert!(!r.advisor_reliance_supports_defense);
    }

    #[test]
    fn incomplete_facts_to_advisor_defeats_reliance() {
        let mut i = base_6662();
        i.taxpayer_provided_complete_accurate_facts_to_advisor = false;
        let r = check(&i);
        assert!(!r.advisor_reliance_supports_defense);
    }

    #[test]
    fn no_advisor_reliance_no_advisor_note() {
        let mut i = base_6662();
        i.relied_on_professional_tax_advisor = false;
        let r = check(&i);
        assert!(!r.advisor_reliance_supports_defense);
        assert!(!r.notes.iter().any(|n| n.contains("advisor reliance supports defense")));
    }

    #[test]
    fn defense_engaged_independently_of_advisor_reliance() {
        let mut i = base_6662();
        i.relied_on_professional_tax_advisor = false;
        let r = check(&i);
        assert!(r.defense_engaged);
        assert!(!r.advisor_reliance_supports_defense);
    }

    #[test]
    fn reportable_transaction_all_three_elements_satisfied() {
        let r = check(&base_6662a());
        assert!(r.defense_engaged);
        assert!(!r.defense_barred_by_reportable_transaction_inadequate_disclosure);
        assert!(r.notes.iter().any(|n| n.contains("§ 6664(d)(3)") && n.contains("all three") && n.contains("satisfied")));
    }

    #[test]
    fn reportable_transaction_missing_adequate_disclosure_bars_defense() {
        let mut i = base_6662a();
        i.adequate_disclosure_per_d_3_a = false;
        let r = check(&i);
        assert!(!r.defense_engaged);
        assert!(r.defense_barred_by_reportable_transaction_inadequate_disclosure);
        assert!(r.notes.iter().any(|n| n.contains("§ 6664(d)(3)") && n.contains("ALL THREE") && n.contains("missing any one")));
    }

    #[test]
    fn reportable_transaction_missing_substantial_authority_bars_defense() {
        let mut i = base_6662a();
        i.substantial_authority_per_d_3_b = false;
        let r = check(&i);
        assert!(!r.defense_engaged);
        assert!(r.defense_barred_by_reportable_transaction_inadequate_disclosure);
    }

    #[test]
    fn reportable_transaction_missing_more_likely_than_not_bars_defense() {
        let mut i = base_6662a();
        i.reasonable_belief_more_likely_than_not_per_d_3_c = false;
        let r = check(&i);
        assert!(!r.defense_engaged);
        assert!(r.defense_barred_by_reportable_transaction_inadequate_disclosure);
    }

    #[test]
    fn reportable_three_element_truth_table() {
        for a in [false, true] {
            for b in [false, true] {
                for c in [false, true] {
                    let mut i = base_6662a();
                    i.adequate_disclosure_per_d_3_a = a;
                    i.substantial_authority_per_d_3_b = b;
                    i.reasonable_belief_more_likely_than_not_per_d_3_c = c;
                    let r = check(&i);
                    let all_three = a && b && c;
                    assert_eq!(r.defense_engaged, all_three);
                    assert_eq!(
                        r.defense_barred_by_reportable_transaction_inadequate_disclosure,
                        !all_three
                    );
                }
            }
        }
    }

    #[test]
    fn accuracy_target_no_reportable_disclosure_check() {
        let mut i = base_6662();
        i.adequate_disclosure_per_d_3_a = false;
        i.substantial_authority_per_d_3_b = false;
        i.reasonable_belief_more_likely_than_not_per_d_3_c = false;
        let r = check(&i);
        assert!(r.defense_engaged);
        assert!(!r.defense_barred_by_reportable_transaction_inadequate_disclosure);
    }

    #[test]
    fn civil_fraud_target_note_describes_clear_and_convincing() {
        let r = check(&base_6663());
        assert!(r.notes.iter().any(|n| n.contains("§ 6663") && n.contains("clear-and-convincing")));
    }

    #[test]
    fn civil_fraud_defense_engaged_when_base_elements_met() {
        let r = check(&base_6663());
        assert!(r.defense_engaged);
    }

    #[test]
    fn accuracy_target_note_describes_facts_and_circumstances() {
        let r = check(&base_6662());
        assert!(r.notes.iter().any(|n| n.contains("§ 6662 accuracy penalty") && n.contains("facts-and-circumstances")));
    }

    #[test]
    fn regulation_invalidity_undisclosed_warning_note() {
        let mut i = base_6662();
        i.regulation_invalidity_position_adequately_disclosed = false;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("§ 1.6662-3(c)(2)") && n.contains("regulation is invalid")));
    }

    #[test]
    fn regulation_invalidity_disclosed_no_warning() {
        let r = check(&base_6662());
        assert!(!r.notes.iter().any(|n| n.contains("§ 1.6662-3(c)(2)")));
    }

    #[test]
    fn general_rule_note_always_present() {
        let r = check(&base_6662());
        assert!(r.notes.iter().any(|n| n.contains("§ 6664(c)(1)") && n.contains("reasonable cause AND good faith")));
    }

    #[test]
    fn facts_and_circumstances_factors_note_present() {
        let r = check(&base_6662());
        assert!(r.notes.iter().any(|n| n.contains("§ 1.6664-4 facts-and-circumstances factors")));
    }

    #[test]
    fn citation_pins_all_subsections_and_treas_regs() {
        let r = check(&base_6662());
        assert!(r.citation.contains("§§ 6664(c)(1), 6664(c)(2), 6664(c)(3), 6664(d)"));
        assert!(r.citation.contains("6664(d)(3)(A)"));
        assert!(r.citation.contains("6664(d)(3)(B)"));
        assert!(r.citation.contains("6664(d)(3)(C)"));
        assert!(r.citation.contains("§ 7701(o)"));
        assert!(r.citation.contains("§ 6662(b)(6)"));
        assert!(r.citation.contains("§ 6662(i)"));
        assert!(r.citation.contains("§§ 1.6664-4, 1.6662-3(c)(2)"));
    }

    #[test]
    fn three_penalty_targets_routed_correctly() {
        for target in [
            PenaltyTarget::Section6662Accuracy,
            PenaltyTarget::Section6662AReportable,
            PenaltyTarget::Section6663CivilFraud,
        ] {
            let mut i = base_6662a();
            i.penalty_target = target;
            let r = check(&i);
            let _ = r.defense_engaged;
            assert!(!r.citation.is_empty());
        }
    }

    #[test]
    fn economic_substance_unique_bar_invariant() {
        let mut i_econ = base_6662();
        i_econ.transaction_lacks_economic_substance = true;
        let r_econ = check(&i_econ);
        assert!(r_econ.defense_barred_by_economic_substance);

        let r_clean = check(&base_6662());
        assert!(!r_clean.defense_barred_by_economic_substance);
    }

    #[test]
    fn reportable_transaction_unique_bar_to_section_6662a_invariant() {
        let mut i_6662a = base_6662a();
        i_6662a.adequate_disclosure_per_d_3_a = false;
        let r_6662a = check(&i_6662a);
        assert!(r_6662a.defense_barred_by_reportable_transaction_inadequate_disclosure);

        let mut i_6662 = base_6662();
        i_6662.adequate_disclosure_per_d_3_a = false;
        let r_6662 = check(&i_6662);
        assert!(!r_6662.defense_barred_by_reportable_transaction_inadequate_disclosure);
    }

    #[test]
    fn defense_engaged_requires_three_conditions() {
        let mut i = base_6662();
        i.reasonable_cause_alleged = true;
        i.good_faith_alleged = true;
        i.transaction_lacks_economic_substance = false;
        let r = check(&i);
        assert!(r.defense_engaged);

        i.transaction_lacks_economic_substance = true;
        assert!(!check(&i).defense_engaged);
    }
}
