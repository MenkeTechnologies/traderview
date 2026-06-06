//! IRC § 7206 — Fraud and false statements. Criminal counterpart
//! to civil fraud penalty under § 6663. Felony with up to 3 years
//! imprisonment plus fines. § 7206(1) is the workhorse criminal
//! tax statute — the "tax perjury" provision applied whenever a
//! taxpayer willfully signs a return under penalty of perjury
//! knowing it to be false. Natural sibling to `section_6663`
//! (civil fraud 75%), `section_6664` (reasonable cause defense),
//! `section_6501` (§ 6501(c)(1) unlimited ASED for fraud), and
//! `section_7491` (burden of proof shifts; criminal cases NOT
//! governed by § 7491 — government bears beyond-reasonable-doubt
//! burden).
//!
//! **Five enumerated offenses**:
//!
//! **§ 7206(1) — Tax perjury (most common)**. Willfully makes
//! and subscribes any return, statement, or other document
//! which contains or is verified by a written declaration that
//! it is made under penalties of perjury, AND which the maker
//! does NOT believe to be true and correct as to every material
//! matter.
//!
//! **§ 7206(2) — Aiding or assisting false document**. Willfully
//! aids or assists in, procures, counsels, or advises the
//! preparation or presentation of any return, statement, or
//! other document which is false or fraudulent as to any
//! material matter. Reaches return preparers, advisors, and
//! third parties even when the taxpayer-signer is innocent.
//!
//! **§ 7206(3) — Fraudulent bonds, permits, and entries**.
//! Simulates or falsely or fraudulently executes any bond,
//! permit, entry, or other document required by the IRS.
//!
//! **§ 7206(4) — Removal or concealment with intent to defraud**.
//! Removes, deposits, or conceals goods or commodities on which
//! tax has been imposed with intent to evade or defeat the tax.
//!
//! **§ 7206(5) — Compromises and closing agreements**. Concealment
//! of property, false statements, or destruction of records in
//! connection with a compromise or closing agreement under
//! § 7121 or § 7122.
//!
//! **§ 7206(1) five-element test** (government bears
//! beyond-reasonable-doubt burden):
//!
//! 1. Defendant MADE AND SUBSCRIBED a return / statement /
//!    other document.
//! 2. Document was FALSE as to a MATERIAL matter.
//! 3. Document contained a written DECLARATION UNDER PENALTIES
//!    OF PERJURY.
//! 4. Defendant DID NOT BELIEVE the document to be true and
//!    correct as to every material matter.
//! 5. Defendant ACTED WILLFULLY with specific intent to violate
//!    the law (Cheek v. United States, 498 U.S. 192 (1991)
//!    good-faith mistake-of-law defense available).
//!
//! All five required; failure of any element defeats prosecution.
//!
//! **Penalties**:
//! - Imprisonment up to 3 YEARS, OR
//! - Fine up to $100,000 individual / $500,000 corporation under
//!   § 7206 itself, but 18 U.S.C. § 3571 (Criminal Fines
//!   Improvement Act) supersedes to $250,000 individual /
//!   $500,000 corporation, OR
//! - BOTH imprisonment and fine, PLUS
//! - Costs of prosecution.
//!
//! **Cheek defense**. Cheek v. United States, 498 U.S. 192
//! (1991) — good-faith misunderstanding of the law (subjective
//! belief) negates willfulness element. NOT a defense for
//! disagreement with the law (constitutional challenges,
//! tax-protester arguments). Cheek is the most important
//! taxpayer defense to § 7206(1) prosecution.
//!
//! **Parallel civil consequences**. § 7206 prosecution may run
//! parallel with § 6663 civil fraud penalty (Spies-Daly
//! doctrine). § 6501(c)(1) UNLIMITED ASED applies once fraud
//! established. § 6501(c)(2) UNLIMITED ASED for willful attempt
//! to evade.
//!
//! **§ 6531 criminal statute of limitations** — 6 years for §
//! 7206(1), 7206(2), 7206(3), 7206(4); 3 years for § 7206(5).
//!
//! Citations: IRC § 7206(1) tax perjury; § 7206(2) aiding /
//! assisting; § 7206(3) fraudulent bonds, permits, entries;
//! § 7206(4) removal / concealment with intent to defraud;
//! § 7206(5) compromises and closing agreements; § 6531
//! criminal statute of limitations; 18 U.S.C. § 3571 Criminal
//! Fines Improvement Act; Cheek v. United States, 498 U.S. 192
//! (1991); IRM 9.1.3 Criminal Statutory Provisions and Common
//! Law; § 6663 (civil fraud counterpart 75%); § 6501(c)(1) /
//! (c)(2) unlimited ASED.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SubsectionTarget {
    /// § 7206(1) — Tax perjury (workhorse statute).
    Section7206_1TaxPerjury,
    /// § 7206(2) — Aiding or assisting false document.
    Section7206_2Aiding,
    /// § 7206(3) — Fraudulent bonds, permits, entries.
    Section7206_3FraudulentDocuments,
    /// § 7206(4) — Removal or concealment with intent to defraud.
    Section7206_4ConcealmentRemoval,
    /// § 7206(5) — Compromises and closing agreements.
    Section7206_5CompromisesClosingAgreements,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    Individual,
    Corporation,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section7206Input {
    pub subsection_target: SubsectionTarget,
    pub entity_type: EntityType,
    // § 7206(1) five-element test:
    pub element_1_made_and_subscribed: bool,
    pub element_2_false_as_to_material_matter: bool,
    pub element_3_declaration_under_penalty_of_perjury: bool,
    pub element_4_did_not_believe_true: bool,
    pub element_5_willful_with_specific_intent: bool,
    /// Whether the Cheek defense is asserted (good-faith
    /// misunderstanding of law negates willfulness).
    pub cheek_defense_asserted: bool,
    /// Whether the Cheek defense is successful (subjective
    /// good-faith belief established).
    pub cheek_defense_successful: bool,
    // § 7206(2) elements:
    pub aided_or_assisted_preparation: bool,
    pub resulting_document_false_or_fraudulent: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section7206Result {
    pub felony_prosecution_authorized: bool,
    pub maximum_imprisonment_years: u32,
    /// Maximum fine in cents (post-18 U.S.C. § 3571 enhancement).
    pub maximum_fine_cents: i64,
    /// Number of § 7206(1) elements satisfied (0-5).
    pub section_1_elements_satisfied_count: u32,
    /// Whether the Cheek good-faith defense is engaged and
    /// successful.
    pub cheek_defense_engaged_and_successful: bool,
    /// Criminal statute of limitations under § 6531 in years.
    pub criminal_sol_years: u32,
    /// Whether § 6663 civil fraud parallel prosecution is
    /// available (Spies-Daly doctrine).
    pub section_6663_civil_parallel_available: bool,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section7206Input) -> Section7206Result {
    let mut notes: Vec<String> = Vec::new();

    let max_fine = match input.entity_type {
        EntityType::Individual => 25_000_000i64,
        EntityType::Corporation => 50_000_000i64,
    };

    let sol_years = match input.subsection_target {
        SubsectionTarget::Section7206_5CompromisesClosingAgreements => 3,
        _ => 6,
    };

    notes.push(
        "§ 7206 — felony tax fraud; up to 3 YEARS imprisonment + fine + costs of prosecution; criminal counterpart to § 6663 civil fraud 75% penalty"
            .to_string(),
    );

    notes.push(
        "18 U.S.C. § 3571 (Criminal Fines Improvement Act) — maximum fines for § 7206 violations: $250,000 individual / $500,000 corporation; supersedes § 7206's original $100,000 individual cap"
            .to_string(),
    );

    notes.push(format!(
        "§ 6531 criminal statute of limitations: {} years for {} subsection",
        sol_years,
        match input.subsection_target {
            SubsectionTarget::Section7206_1TaxPerjury => "§ 7206(1) tax perjury",
            SubsectionTarget::Section7206_2Aiding => "§ 7206(2) aiding or assisting",
            SubsectionTarget::Section7206_3FraudulentDocuments => "§ 7206(3) fraudulent documents",
            SubsectionTarget::Section7206_4ConcealmentRemoval => "§ 7206(4) concealment / removal",
            SubsectionTarget::Section7206_5CompromisesClosingAgreements =>
                "§ 7206(5) compromises and closing agreements",
        }
    ));

    let cheek_engaged = input.cheek_defense_asserted && input.cheek_defense_successful;
    if input.cheek_defense_asserted {
        notes.push(
            "Cheek v. United States, 498 U.S. 192 (1991) — good-faith misunderstanding of law (subjective belief) negates willfulness; NOT a defense for disagreement with the law (constitutional challenges, tax-protester arguments)"
                .to_string(),
        );
        if cheek_engaged {
            notes.push(
                "Cheek defense SUCCESSFUL — subjective good-faith belief established; willfulness element of § 7206(1) defeated"
                    .to_string(),
            );
        } else {
            notes.push(
                "Cheek defense ASSERTED but NOT successful — government has rebutted subjective good-faith belief"
                    .to_string(),
            );
        }
    }

    let (elements_count, felony_authorized) = match input.subsection_target {
        SubsectionTarget::Section7206_1TaxPerjury => {
            let count = [
                input.element_1_made_and_subscribed,
                input.element_2_false_as_to_material_matter,
                input.element_3_declaration_under_penalty_of_perjury,
                input.element_4_did_not_believe_true,
                input.element_5_willful_with_specific_intent,
            ]
            .iter()
            .filter(|&&b| b)
            .count() as u32;

            let all_five = count == 5;
            let willfulness_defeated_by_cheek = cheek_engaged;

            notes.push(format!(
                "§ 7206(1) five-element test: {}/5 satisfied — (1) made and subscribed + (2) false as to material matter + (3) declaration under penalties of perjury + (4) did not believe true + (5) willful with specific intent to violate law",
                count
            ));

            if all_five && !willfulness_defeated_by_cheek {
                notes.push(
                    "§ 7206(1) — all five elements satisfied; felony prosecution AUTHORIZED"
                        .to_string(),
                );
            } else if !all_five {
                notes.push(
                    "§ 7206(1) — government has not established all five elements beyond reasonable doubt; prosecution NOT authorized"
                        .to_string(),
                );
            } else if willfulness_defeated_by_cheek {
                notes.push(
                    "§ 7206(1) — Cheek defense defeats willfulness element; prosecution NOT authorized despite other elements satisfied"
                        .to_string(),
                );
            }

            (count, all_five && !willfulness_defeated_by_cheek)
        }
        SubsectionTarget::Section7206_2Aiding => {
            let elements_met = input.aided_or_assisted_preparation
                && input.resulting_document_false_or_fraudulent
                && input.element_5_willful_with_specific_intent;
            let count = [
                input.aided_or_assisted_preparation,
                input.resulting_document_false_or_fraudulent,
                input.element_5_willful_with_specific_intent,
            ]
            .iter()
            .filter(|&&b| b)
            .count() as u32;

            notes.push(
                "§ 7206(2) — aids, assists, procures, counsels, or advises preparation of return or document false or fraudulent as to any material matter; reaches return preparers, advisors, third parties EVEN WHEN taxpayer-signer is innocent"
                    .to_string(),
            );

            (count, elements_met && !cheek_engaged)
        }
        _ => {
            notes.push(
                "§ 7206(3) / (4) / (5) — felony tax fraud variants apply to fraudulent bonds/permits, concealment/removal of taxed goods, or fraud in connection with compromise/closing agreements"
                    .to_string(),
            );
            (0, false)
        }
    };

    notes.push(
        "Spies-Daly doctrine — criminal prosecution under § 7206 may PROCEED IN PARALLEL with civil fraud penalty under § 6663 75%; double jeopardy does NOT bar civil penalty after criminal conviction (or acquittal)"
            .to_string(),
    );

    notes.push(
        "§ 6501(c)(1) UNLIMITED ASED applies once fraud established under § 7206 — IRS may assess at any time; § 6501(c)(2) UNLIMITED ASED for willful attempt to evade"
            .to_string(),
    );

    notes.push(
        "§ 7491 burden of proof shifts do NOT apply to criminal prosecutions; government bears BEYOND REASONABLE DOUBT burden on each element"
            .to_string(),
    );

    notes.push(
        "IRM 9.1.3 — Criminal Statutory Provisions and Common Law procedural manual governs IRS Criminal Investigation (CI) referral process to DOJ Tax Division"
            .to_string(),
    );

    Section7206Result {
        felony_prosecution_authorized: felony_authorized,
        maximum_imprisonment_years: 3,
        maximum_fine_cents: max_fine,
        section_1_elements_satisfied_count: elements_count,
        cheek_defense_engaged_and_successful: cheek_engaged,
        criminal_sol_years: sol_years,
        section_6663_civil_parallel_available: true,
        citation: "IRC §§ 7206(1), 7206(2), 7206(3), 7206(4), 7206(5), 6531, 6663, 6501(c)(1), 6501(c)(2); 18 U.S.C. § 3571 (Criminal Fines Improvement Act); Cheek v. United States, 498 U.S. 192 (1991); IRM 9.1.3",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn section_1_full() -> Section7206Input {
        Section7206Input {
            subsection_target: SubsectionTarget::Section7206_1TaxPerjury,
            entity_type: EntityType::Individual,
            element_1_made_and_subscribed: true,
            element_2_false_as_to_material_matter: true,
            element_3_declaration_under_penalty_of_perjury: true,
            element_4_did_not_believe_true: true,
            element_5_willful_with_specific_intent: true,
            cheek_defense_asserted: false,
            cheek_defense_successful: false,
            aided_or_assisted_preparation: false,
            resulting_document_false_or_fraudulent: false,
        }
    }

    fn section_2_full() -> Section7206Input {
        let mut i = section_1_full();
        i.subsection_target = SubsectionTarget::Section7206_2Aiding;
        i.aided_or_assisted_preparation = true;
        i.resulting_document_false_or_fraudulent = true;
        i
    }

    #[test]
    fn section_1_all_five_elements_authorizes_prosecution() {
        let r = check(&section_1_full());
        assert!(r.felony_prosecution_authorized);
        assert_eq!(r.section_1_elements_satisfied_count, 5);
        assert_eq!(r.maximum_imprisonment_years, 3);
    }

    #[test]
    fn section_1_missing_any_element_no_prosecution() {
        let mut i = section_1_full();
        i.element_2_false_as_to_material_matter = false;
        let r = check(&i);
        assert!(!r.felony_prosecution_authorized);
        assert_eq!(r.section_1_elements_satisfied_count, 4);
    }

    #[test]
    fn section_1_five_element_truth_table() {
        for e1 in [false, true] {
            for e2 in [false, true] {
                for e3 in [false, true] {
                    for e4 in [false, true] {
                        for e5 in [false, true] {
                            let mut i = section_1_full();
                            i.element_1_made_and_subscribed = e1;
                            i.element_2_false_as_to_material_matter = e2;
                            i.element_3_declaration_under_penalty_of_perjury = e3;
                            i.element_4_did_not_believe_true = e4;
                            i.element_5_willful_with_specific_intent = e5;
                            let r = check(&i);
                            let all_five = e1 && e2 && e3 && e4 && e5;
                            assert_eq!(r.felony_prosecution_authorized, all_five);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn cheek_defense_successful_defeats_prosecution() {
        let mut i = section_1_full();
        i.cheek_defense_asserted = true;
        i.cheek_defense_successful = true;
        let r = check(&i);
        assert!(!r.felony_prosecution_authorized);
        assert!(r.cheek_defense_engaged_and_successful);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Cheek defense SUCCESSFUL")));
    }

    #[test]
    fn cheek_defense_unsuccessful_does_not_defeat() {
        let mut i = section_1_full();
        i.cheek_defense_asserted = true;
        i.cheek_defense_successful = false;
        let r = check(&i);
        assert!(r.felony_prosecution_authorized);
        assert!(!r.cheek_defense_engaged_and_successful);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Cheek defense ASSERTED but NOT successful")));
    }

    #[test]
    fn cheek_not_asserted_no_cheek_engagement() {
        let r = check(&section_1_full());
        assert!(!r.cheek_defense_engaged_and_successful);
        assert!(!r.notes.iter().any(|n| n.contains("Cheek defense ASSERTED")));
    }

    #[test]
    fn cheek_case_note_describes_subjective_belief_test() {
        let mut i = section_1_full();
        i.cheek_defense_asserted = true;
        let r = check(&i);
        assert!(r.notes.iter().any(
            |n| n.contains("Cheek v. United States, 498 U.S. 192 (1991)")
                && n.contains("subjective belief")
        ));
    }

    #[test]
    fn individual_max_fine_250k() {
        let r = check(&section_1_full());
        assert_eq!(r.maximum_fine_cents, 25_000_000);
    }

    #[test]
    fn corporation_max_fine_500k() {
        let mut i = section_1_full();
        i.entity_type = EntityType::Corporation;
        let r = check(&i);
        assert_eq!(r.maximum_fine_cents, 50_000_000);
    }

    #[test]
    fn maximum_imprisonment_always_3_years() {
        for target in [
            SubsectionTarget::Section7206_1TaxPerjury,
            SubsectionTarget::Section7206_2Aiding,
            SubsectionTarget::Section7206_3FraudulentDocuments,
            SubsectionTarget::Section7206_4ConcealmentRemoval,
            SubsectionTarget::Section7206_5CompromisesClosingAgreements,
        ] {
            let mut i = section_1_full();
            i.subsection_target = target;
            assert_eq!(check(&i).maximum_imprisonment_years, 3);
        }
    }

    #[test]
    fn section_7206_1_2_3_4_six_year_sol() {
        for target in [
            SubsectionTarget::Section7206_1TaxPerjury,
            SubsectionTarget::Section7206_2Aiding,
            SubsectionTarget::Section7206_3FraudulentDocuments,
            SubsectionTarget::Section7206_4ConcealmentRemoval,
        ] {
            let mut i = section_1_full();
            i.subsection_target = target;
            assert_eq!(check(&i).criminal_sol_years, 6);
        }
    }

    #[test]
    fn section_7206_5_three_year_sol() {
        let mut i = section_1_full();
        i.subsection_target = SubsectionTarget::Section7206_5CompromisesClosingAgreements;
        assert_eq!(check(&i).criminal_sol_years, 3);
    }

    #[test]
    fn section_2_aiding_full_elements_authorizes() {
        let r = check(&section_2_full());
        assert!(r.felony_prosecution_authorized);
        assert!(r.notes.iter().any(
            |n| n.contains("§ 7206(2)") && n.contains("EVEN WHEN taxpayer-signer is innocent")
        ));
    }

    #[test]
    fn section_2_aiding_missing_aid_no_prosecution() {
        let mut i = section_2_full();
        i.aided_or_assisted_preparation = false;
        let r = check(&i);
        assert!(!r.felony_prosecution_authorized);
    }

    #[test]
    fn section_2_aiding_missing_willfulness_no_prosecution() {
        let mut i = section_2_full();
        i.element_5_willful_with_specific_intent = false;
        let r = check(&i);
        assert!(!r.felony_prosecution_authorized);
    }

    #[test]
    fn section_2_cheek_defeats_aiding() {
        let mut i = section_2_full();
        i.cheek_defense_asserted = true;
        i.cheek_defense_successful = true;
        let r = check(&i);
        assert!(!r.felony_prosecution_authorized);
    }

    #[test]
    fn section_6663_civil_parallel_always_available() {
        let r = check(&section_1_full());
        assert!(r.section_6663_civil_parallel_available);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Spies-Daly doctrine") && n.contains("PARALLEL")));
    }

    #[test]
    fn unlimited_ased_cross_reference_note() {
        let r = check(&section_1_full());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6501(c)(1) UNLIMITED ASED")));
    }

    #[test]
    fn section_7491_burden_shifts_excluded_note() {
        let r = check(&section_1_full());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 7491") && n.contains("BEYOND REASONABLE DOUBT")));
    }

    #[test]
    fn cfia_18_usc_3571_supersedes_note() {
        let r = check(&section_1_full());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("18 U.S.C. § 3571") && n.contains("$250,000")));
    }

    #[test]
    fn irm_9_1_3_note_present() {
        let r = check(&section_1_full());
        assert!(r.notes.iter().any(|n| n.contains("IRM 9.1.3")));
    }

    #[test]
    fn citation_pins_all_subsections() {
        let r = check(&section_1_full());
        assert!(r
            .citation
            .contains("§§ 7206(1), 7206(2), 7206(3), 7206(4), 7206(5)"));
        assert!(r.citation.contains("6531"));
        assert!(r.citation.contains("6663"));
        assert!(r.citation.contains("6501(c)(1)"));
        assert!(r.citation.contains("6501(c)(2)"));
        assert!(r.citation.contains("18 U.S.C. § 3571"));
        assert!(r.citation.contains("Cheek v. United States"));
        assert!(r.citation.contains("IRM 9.1.3"));
    }

    #[test]
    fn five_subsections_routed_correctly() {
        for target in [
            SubsectionTarget::Section7206_1TaxPerjury,
            SubsectionTarget::Section7206_2Aiding,
            SubsectionTarget::Section7206_3FraudulentDocuments,
            SubsectionTarget::Section7206_4ConcealmentRemoval,
            SubsectionTarget::Section7206_5CompromisesClosingAgreements,
        ] {
            let mut i = section_1_full();
            i.subsection_target = target;
            i.aided_or_assisted_preparation = true;
            i.resulting_document_false_or_fraudulent = true;
            let r = check(&i);
            let _ = r.felony_prosecution_authorized;
            assert!(!r.citation.is_empty());
        }
    }

    #[test]
    fn elements_count_zero_when_no_elements() {
        let mut i = section_1_full();
        i.element_1_made_and_subscribed = false;
        i.element_2_false_as_to_material_matter = false;
        i.element_3_declaration_under_penalty_of_perjury = false;
        i.element_4_did_not_believe_true = false;
        i.element_5_willful_with_specific_intent = false;
        let r = check(&i);
        assert_eq!(r.section_1_elements_satisfied_count, 0);
        assert!(!r.felony_prosecution_authorized);
    }

    #[test]
    fn elements_count_increments_per_element() {
        let mut i = section_1_full();
        i.element_1_made_and_subscribed = false;
        i.element_2_false_as_to_material_matter = false;
        i.element_3_declaration_under_penalty_of_perjury = false;
        i.element_4_did_not_believe_true = false;
        i.element_5_willful_with_specific_intent = false;
        assert_eq!(check(&i).section_1_elements_satisfied_count, 0);

        i.element_1_made_and_subscribed = true;
        assert_eq!(check(&i).section_1_elements_satisfied_count, 1);

        i.element_2_false_as_to_material_matter = true;
        i.element_3_declaration_under_penalty_of_perjury = true;
        assert_eq!(check(&i).section_1_elements_satisfied_count, 3);

        i.element_4_did_not_believe_true = true;
        i.element_5_willful_with_specific_intent = true;
        assert_eq!(check(&i).section_1_elements_satisfied_count, 5);
    }

    #[test]
    fn section_1_five_element_note_present() {
        let r = check(&section_1_full());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 7206(1) five-element test") && n.contains("5/5")));
    }

    #[test]
    fn section_2_aiding_element_note_describes_reach() {
        let r = check(&section_2_full());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 7206(2)") && n.contains("preparers, advisors, third parties")));
    }

    #[test]
    fn cheek_only_defeats_when_successful_and_asserted() {
        for asserted in [false, true] {
            for successful in [false, true] {
                let mut i = section_1_full();
                i.cheek_defense_asserted = asserted;
                i.cheek_defense_successful = successful;
                let r = check(&i);
                let cheek_defeats = asserted && successful;
                assert_eq!(r.cheek_defense_engaged_and_successful, cheek_defeats);
                assert_eq!(r.felony_prosecution_authorized, !cheek_defeats);
            }
        }
    }

    #[test]
    fn sol_truth_table_per_subsection() {
        let pairs = [
            (SubsectionTarget::Section7206_1TaxPerjury, 6u32),
            (SubsectionTarget::Section7206_2Aiding, 6),
            (SubsectionTarget::Section7206_3FraudulentDocuments, 6),
            (SubsectionTarget::Section7206_4ConcealmentRemoval, 6),
            (
                SubsectionTarget::Section7206_5CompromisesClosingAgreements,
                3,
            ),
        ];
        for (target, expected_sol) in pairs {
            let mut i = section_1_full();
            i.subsection_target = target;
            assert_eq!(check(&i).criminal_sol_years, expected_sol);
        }
    }
}
