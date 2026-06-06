//! IRC § 7207 — Fraudulent returns, statements, or other
//! documents. Criminal MISDEMEANOR (1-year imprisonment cap) for
//! willful and knowing delivery or disclosure to the IRS of a
//! false or fraudulent document (regardless of whether signed
//! under penalties of perjury). Pairs with `section_7206`
//! (felony perjury, signed under penalties of perjury) and
//! `section_7434` (civil damages for fraudulent information
//! return). Completes the false-documents criminal statute
//! trio with § 7206 + § 7207.
//!
//! **Three-element test** (BEYOND REASONABLE DOUBT):
//!
//! 1. DELIVERY OR DISCLOSURE to an IRS officer or employee of
//!    any list, return, account, statement, or OTHER DOCUMENT
//! 2. Document was FALSE OR FRAUDULENT as to a MATERIAL MATTER
//! 3. Done WILLFULLY or with knowledge of the falsity or fraud
//!
//! All three required; failure of any defeats prosecution.
//!
//! **Distinction from § 7206**:
//!
//! - § 7206(1) is a FELONY (3-year imprisonment) for documents
//!   signed UNDER PENALTIES OF PERJURY (e.g., tax returns).
//! - § 7207 is a MISDEMEANOR (1-year imprisonment) for ANY
//!   false document delivered to IRS, regardless of perjury
//!   verification. Broader scope but lower penalty.
//!
//! Government typically charges § 7207 when: (a) computed tax
//! deficiency is de minimis, (b) document was NOT signed under
//! penalties of perjury, or (c) prosecutorial discretion favors
//! misdemeanor over felony.
//!
//! **Penalties**:
//!
//! - Imprisonment up to **1 YEAR** (misdemeanor)
//! - Fine up to $10,000 individual / $50,000 corporation under
//!   § 7207 original text
//! - 18 U.S.C. § 3571 (Criminal Fines Improvement Act)
//!   supersedes to $100,000 individual / $200,000 corporation
//! - BOTH imprisonment and fine permitted
//! - Costs of prosecution
//!
//! **Cheek defense** — Cheek v. United States, 498 U.S. 192
//! (1991) good-faith subjective misunderstanding of law negates
//! willfulness element. Same defense as § 7201 / § 7203 /
//! § 7206.
//!
//! **§ 6531 SOL — 6 YEARS** for § 7207 per § 6531(2)
//! enumerated 6-year offenses (see `section_6531`).
//!
//! **Typical § 7207 fact patterns**:
//!
//! - Taxpayer delivers fabricated receipts to IRS during audit
//!   to substantiate claimed deductions
//! - Taxpayer presents altered Schedule K-1 to IRS examiner
//! - Tax preparer delivers fraudulent supporting documents on
//!   client's behalf
//! - Taxpayer files false Form 433 (collection information
//!   statement) to obtain installment agreement
//!
//! **Cross-references**:
//!
//! - § 7206(1) tax perjury (felony alternative when document
//!   signed under penalties of perjury)
//! - § 7434 civil damages for fraudulent information return
//!   (third-party victim civil cause of action)
//! - § 7212 obstruction of administration (when conduct also
//!   obstructs IRS examination)
//! - § 6663 civil fraud 75% penalty (parallel civil track)
//! - § 6501(c)(1) UNLIMITED ASED for fraud
//!
//! Citations: IRC § 7207; § 6531 6-year SOL; § 7206 felony
//! perjury alternative; § 7434 civil damages parallel; § 7212
//! obstruction; § 6663 civil fraud 75%; § 6501(c)(1) unlimited
//! ASED; 18 U.S.C. § 3571 Criminal Fines Improvement Act;
//! Cheek v. United States, 498 U.S. 192 (1991); IRM 9.1.3.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    Individual,
    Corporation,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section7207Input {
    pub entity_type: EntityType,
    /// Element 1 — delivery or disclosure to IRS officer or
    /// employee of any list / return / account / statement /
    /// other document.
    pub element_1_delivery_or_disclosure_to_irs: bool,
    /// Element 2 — document was false or fraudulent as to a
    /// material matter.
    pub element_2_false_or_fraudulent_material_matter: bool,
    /// Element 3 — done willfully or with knowledge of falsity
    /// or fraud.
    pub element_3_willful_or_with_knowledge: bool,
    /// Whether the document was signed under penalties of
    /// perjury (triggers § 7206 felony alternative).
    pub document_signed_under_penalties_of_perjury: bool,
    /// Whether the computed tax deficiency is de minimis (favors
    /// § 7207 misdemeanor charging over § 7206 felony).
    pub computed_tax_deficiency_de_minimis: bool,
    /// Whether the Cheek defense is asserted.
    pub cheek_defense_asserted: bool,
    /// Whether the Cheek defense is successful.
    pub cheek_defense_successful: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section7207Result {
    pub misdemeanor_prosecution_authorized: bool,
    pub section_7206_felony_alternative_available: bool,
    pub maximum_imprisonment_years: u32,
    pub maximum_fine_cents: i64,
    pub three_elements_satisfied_count: u32,
    pub cheek_defense_engaged_and_successful: bool,
    pub criminal_sol_years: u32,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section7207Input) -> Section7207Result {
    let mut notes: Vec<String> = Vec::new();

    let max_fine = match input.entity_type {
        EntityType::Individual => 10_000_000i64,
        EntityType::Corporation => 20_000_000i64,
    };

    notes.push(
        "§ 7207 — criminal MISDEMEANOR (1-year imprisonment cap) for willful and knowing delivery or disclosure to IRS of false or fraudulent document; broader scope than § 7206 (covers documents NOT signed under penalties of perjury) but lower penalty"
            .to_string(),
    );

    notes.push(
        "18 U.S.C. § 3571 (Criminal Fines Improvement Act) — maximum fines for § 7207: $100,000 individual / $200,000 corporation; supersedes § 7207's original $10,000 individual / $50,000 corporation caps"
            .to_string(),
    );

    notes.push(
        "§ 6531(2) — 6-year criminal SOL for § 7207 (enumerated 6-year offense; not general 3-year)"
            .to_string(),
    );

    let cheek_engaged = input.cheek_defense_asserted && input.cheek_defense_successful;
    if input.cheek_defense_asserted {
        notes.push(
            "Cheek v. United States, 498 U.S. 192 (1991) — genuine good-faith subjective belief negates willfulness element; subjective belief test; NOT a defense for disagreement with the law"
                .to_string(),
        );
        if cheek_engaged {
            notes.push(
                "Cheek defense SUCCESSFUL — willfulness element of § 7207 defeated".to_string(),
            );
        }
    }

    let count = [
        input.element_1_delivery_or_disclosure_to_irs,
        input.element_2_false_or_fraudulent_material_matter,
        input.element_3_willful_or_with_knowledge,
    ]
    .iter()
    .filter(|&&b| b)
    .count() as u32;

    let all_three = count == 3;
    let misdemeanor_authorized = all_three && !cheek_engaged;

    notes.push(format!(
        "§ 7207 three-element test: {}/3 satisfied — (1) delivery or disclosure to IRS + (2) false or fraudulent as to material matter + (3) willfully or with knowledge of falsity",
        count
    ));

    let felony_alternative =
        all_three && !cheek_engaged && input.document_signed_under_penalties_of_perjury;

    if felony_alternative {
        notes.push(
            "§ 7206(1) FELONY alternative available — document signed under penalties of perjury triggers § 7206 perjury statute (3-year felony + $250,000 individual / $500,000 corporation fine); prosecutor's discretion to charge either § 7207 or § 7206(1)"
                .to_string(),
        );
    }

    if input.computed_tax_deficiency_de_minimis {
        notes.push(
            "§ 7207 typically charged when computed tax deficiency is DE MINIMIS; prosecutorial discretion favors misdemeanor over § 7206 felony in low-loss cases"
                .to_string(),
        );
    }

    if misdemeanor_authorized {
        notes.push(
            "§ 7207 — misdemeanor prosecution AUTHORIZED (1-year imprisonment + fine + costs of prosecution)"
                .to_string(),
        );
    } else if !all_three {
        notes.push(
            "§ 7207 — government has not established all three elements beyond reasonable doubt; prosecution NOT authorized"
                .to_string(),
        );
    } else if cheek_engaged {
        notes.push(
            "§ 7207 — Cheek defense defeats willfulness element; prosecution NOT authorized despite other elements satisfied"
                .to_string(),
        );
    }

    notes.push(
        "Spies-Daly parallel civil consequences — § 7434 civil damages for fraudulent information return + § 6663 civil fraud 75% penalty + § 6501(c)(1) UNLIMITED ASED for fraud"
            .to_string(),
    );

    notes.push(
        "typical § 7207 fact patterns: fabricated receipts during audit + altered Schedule K-1 + fraudulent supporting documents + false Form 433 collection information statement"
            .to_string(),
    );

    notes.push(
        "IRM 9.1.3 — Criminal Statutory Provisions and Common Law procedural manual".to_string(),
    );

    Section7207Result {
        misdemeanor_prosecution_authorized: misdemeanor_authorized,
        section_7206_felony_alternative_available: felony_alternative,
        maximum_imprisonment_years: 1,
        maximum_fine_cents: max_fine,
        three_elements_satisfied_count: count,
        cheek_defense_engaged_and_successful: cheek_engaged,
        criminal_sol_years: 6,
        citation: "IRC §§ 7207, 6531, 7206, 7434, 7212, 6663, 6501(c)(1); 18 U.S.C. § 3571 (Criminal Fines Improvement Act); Cheek v. United States, 498 U.S. 192 (1991); IRM 9.1.3",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full_violation() -> Section7207Input {
        Section7207Input {
            entity_type: EntityType::Individual,
            element_1_delivery_or_disclosure_to_irs: true,
            element_2_false_or_fraudulent_material_matter: true,
            element_3_willful_or_with_knowledge: true,
            document_signed_under_penalties_of_perjury: false,
            computed_tax_deficiency_de_minimis: false,
            cheek_defense_asserted: false,
            cheek_defense_successful: false,
        }
    }

    #[test]
    fn full_three_elements_authorizes_misdemeanor() {
        let r = check(&full_violation());
        assert!(r.misdemeanor_prosecution_authorized);
        assert_eq!(r.three_elements_satisfied_count, 3);
        assert_eq!(r.maximum_imprisonment_years, 1);
    }

    #[test]
    fn individual_max_fine_100k() {
        let r = check(&full_violation());
        assert_eq!(r.maximum_fine_cents, 10_000_000);
    }

    #[test]
    fn corporation_max_fine_200k() {
        let mut i = full_violation();
        i.entity_type = EntityType::Corporation;
        let r = check(&i);
        assert_eq!(r.maximum_fine_cents, 20_000_000);
    }

    #[test]
    fn missing_delivery_element_defeats() {
        let mut i = full_violation();
        i.element_1_delivery_or_disclosure_to_irs = false;
        let r = check(&i);
        assert!(!r.misdemeanor_prosecution_authorized);
    }

    #[test]
    fn missing_falsity_element_defeats() {
        let mut i = full_violation();
        i.element_2_false_or_fraudulent_material_matter = false;
        let r = check(&i);
        assert!(!r.misdemeanor_prosecution_authorized);
    }

    #[test]
    fn missing_willfulness_element_defeats() {
        let mut i = full_violation();
        i.element_3_willful_or_with_knowledge = false;
        let r = check(&i);
        assert!(!r.misdemeanor_prosecution_authorized);
    }

    #[test]
    fn three_element_truth_table() {
        for e1 in [false, true] {
            for e2 in [false, true] {
                for e3 in [false, true] {
                    let mut i = full_violation();
                    i.element_1_delivery_or_disclosure_to_irs = e1;
                    i.element_2_false_or_fraudulent_material_matter = e2;
                    i.element_3_willful_or_with_knowledge = e3;
                    let r = check(&i);
                    let all_three = e1 && e2 && e3;
                    assert_eq!(r.misdemeanor_prosecution_authorized, all_three);
                }
            }
        }
    }

    #[test]
    fn cheek_defense_successful_defeats_prosecution() {
        let mut i = full_violation();
        i.cheek_defense_asserted = true;
        i.cheek_defense_successful = true;
        let r = check(&i);
        assert!(!r.misdemeanor_prosecution_authorized);
        assert!(r.cheek_defense_engaged_and_successful);
    }

    #[test]
    fn cheek_only_defeats_when_successful_and_asserted() {
        for asserted in [false, true] {
            for successful in [false, true] {
                let mut i = full_violation();
                i.cheek_defense_asserted = asserted;
                i.cheek_defense_successful = successful;
                let r = check(&i);
                let cheek_defeats = asserted && successful;
                assert_eq!(r.cheek_defense_engaged_and_successful, cheek_defeats);
                assert_eq!(r.misdemeanor_prosecution_authorized, !cheek_defeats);
            }
        }
    }

    #[test]
    fn signed_under_perjury_triggers_7206_alternative() {
        let mut i = full_violation();
        i.document_signed_under_penalties_of_perjury = true;
        let r = check(&i);
        assert!(r.section_7206_felony_alternative_available);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 7206(1) FELONY alternative")
                && n.contains("perjury statute")
                && n.contains("$250,000")));
    }

    #[test]
    fn not_signed_under_perjury_no_7206_alternative() {
        let r = check(&full_violation());
        assert!(!r.section_7206_felony_alternative_available);
    }

    #[test]
    fn de_minimis_deficiency_note_present() {
        let mut i = full_violation();
        i.computed_tax_deficiency_de_minimis = true;
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("DE MINIMIS") && n.contains("misdemeanor over § 7206 felony")));
    }

    #[test]
    fn criminal_sol_6_years() {
        let r = check(&full_violation());
        assert_eq!(r.criminal_sol_years, 6);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6531(2)") && n.contains("6-year")));
    }

    #[test]
    fn cfia_supersedes_fine_caps_note() {
        let r = check(&full_violation());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("18 U.S.C. § 3571") && n.contains("$100,000")));
    }

    #[test]
    fn spies_daly_parallel_civil_note_present() {
        let r = check(&full_violation());
        assert!(r.notes.iter().any(|n| n.contains("Spies-Daly")
            && n.contains("§ 7434")
            && n.contains("§ 6663")
            && n.contains("§ 6501(c)(1)")));
    }

    #[test]
    fn typical_fact_patterns_note_present() {
        let r = check(&full_violation());
        assert!(r.notes.iter().any(|n| n.contains("fabricated receipts")
            && n.contains("altered Schedule K-1")
            && n.contains("Form 433")));
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&full_violation());
        assert!(r.citation.contains("§§ 7207, 6531, 7206, 7434"));
        assert!(r.citation.contains("7212"));
        assert!(r.citation.contains("6663"));
        assert!(r.citation.contains("6501(c)(1)"));
        assert!(r.citation.contains("18 U.S.C. § 3571"));
        assert!(r.citation.contains("Cheek v. United States"));
        assert!(r.citation.contains("IRM 9.1.3"));
    }

    #[test]
    fn maximum_imprisonment_always_1_year() {
        for signed in [false, true] {
            let mut i = full_violation();
            i.document_signed_under_penalties_of_perjury = signed;
            let r = check(&i);
            assert_eq!(r.maximum_imprisonment_years, 1);
        }
    }

    #[test]
    fn distinction_from_7206_note_describes_perjury_scope() {
        let r = check(&full_violation());
        assert!(r.notes.iter().any(|n| n.contains("§ 7207")
            && n.contains("broader scope")
            && n.contains("NOT signed under penalties of perjury")));
    }

    #[test]
    fn three_element_count_increments_per_element() {
        let mut i = full_violation();
        i.element_1_delivery_or_disclosure_to_irs = false;
        i.element_2_false_or_fraudulent_material_matter = false;
        i.element_3_willful_or_with_knowledge = false;
        assert_eq!(check(&i).three_elements_satisfied_count, 0);

        i.element_1_delivery_or_disclosure_to_irs = true;
        assert_eq!(check(&i).three_elements_satisfied_count, 1);

        i.element_2_false_or_fraudulent_material_matter = true;
        assert_eq!(check(&i).three_elements_satisfied_count, 2);

        i.element_3_willful_or_with_knowledge = true;
        assert_eq!(check(&i).three_elements_satisfied_count, 3);
    }

    #[test]
    fn misdemeanor_authorized_note_describes_pathway() {
        let r = check(&full_violation());
        assert!(r.notes.iter().any(
            |n| n.contains("§ 7207 — misdemeanor prosecution AUTHORIZED") && n.contains("1-year")
        ));
    }

    #[test]
    fn felony_alternative_only_when_signed_and_all_three_elements() {
        let mut i = full_violation();
        i.document_signed_under_penalties_of_perjury = true;
        let r = check(&i);
        assert!(r.section_7206_felony_alternative_available);

        i.element_1_delivery_or_disclosure_to_irs = false;
        let r2 = check(&i);
        assert!(!r2.section_7206_felony_alternative_available);
    }

    #[test]
    fn cheek_engaged_disables_felony_alternative() {
        let mut i = full_violation();
        i.document_signed_under_penalties_of_perjury = true;
        i.cheek_defense_asserted = true;
        i.cheek_defense_successful = true;
        let r = check(&i);
        assert!(!r.section_7206_felony_alternative_available);
        assert!(!r.misdemeanor_prosecution_authorized);
    }

    #[test]
    fn irm_9_1_3_note_present() {
        let r = check(&full_violation());
        assert!(r.notes.iter().any(|n| n.contains("IRM 9.1.3")));
    }

    #[test]
    fn cheek_case_note_describes_subjective_belief() {
        let mut i = full_violation();
        i.cheek_defense_asserted = true;
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Cheek v. United States") && n.contains("subjective belief")));
    }

    #[test]
    fn prosecution_not_authorized_when_elements_missing_note() {
        let mut i = full_violation();
        i.element_1_delivery_or_disclosure_to_irs = false;
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("government has not established all three")
                && n.contains("prosecution NOT authorized")));
    }
}
