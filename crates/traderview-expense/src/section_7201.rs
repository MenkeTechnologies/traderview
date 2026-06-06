//! IRC § 7201 — Attempt to evade or defeat tax. The apex criminal
//! tax statute. FELONY with up to 5 years imprisonment + fines.
//! Pairs with `section_7206` (fraud and false statements / tax
//! perjury — 3-year felony) and `section_6663` (civil fraud 75%
//! penalty). Natural sibling to `section_6501` (§ 6501(c)(1)
//! unlimited ASED for fraud), `section_6664` (reasonable cause
//! defense), and `section_7491` (burden shifts excluded for
//! criminal cases).
//!
//! Trader-relevant when aggressive § 1256 mark-to-market positions,
//! § 988 currency reclassifications, § 1202 QSBS holding periods,
//! § 475(f) trader-tax-status claims involve any affirmative
//! conduct to conceal, mislead, or destroy records.
//!
//! **§ 7201 four-element test** (government bears BEYOND
//! REASONABLE DOUBT burden):
//!
//! 1. Existence of a TAX DEFICIENCY (additional tax owed).
//! 2. WILLFULNESS — voluntary intentional violation of a known
//!    legal duty.
//! 3. An AFFIRMATIVE ACT of evasion (Spies doctrine — omissions
//!    alone insufficient; mere failure to file does NOT
//!    constitute § 7201 evasion).
//! 4. SUBSTANTIAL amount of tax owed.
//!
//! All four required; failure of any defeats prosecution.
//!
//! **Spies v. United States, 317 U.S. 492 (1943) — affirmative
//! act doctrine**. The hallmark of § 7201 is conduct "the likely
//! effect of which would be to mislead or to conceal." Spies
//! enumerates seven specific affirmative-act indicia:
//!
//! - Keeping a DOUBLE SET OF BOOKS
//! - Making FALSE ENTRIES OR ALTERATIONS
//! - Making FALSE INVOICES OR DOCUMENTS
//! - DESTRUCTION OF BOOKS OR RECORDS
//! - CONCEALMENT OF ASSETS
//! - COVERING UP SOURCES OF INCOME
//! - Handling one's affairs to avoid making the records usual
//!   in transactions of the kind
//!
//! Presence of any Spies indicium supports the affirmative-act
//! element. Mere failure to file or pay (omission) does NOT.
//!
//! **Two forms of § 7201 evasion** (Sansone v. United States,
//! 380 U.S. 343 (1965)):
//!
//! - **Evasion of ASSESSMENT** — taxpayer takes affirmative act
//!   to prevent IRS from determining the correct tax (false
//!   return, hidden income).
//! - **Evasion of PAYMENT** — taxpayer takes affirmative act
//!   AFTER the tax has been assessed to prevent IRS from
//!   collecting (concealment of assets, transfers to nominees).
//!
//! Both forms covered by § 7201; charging instrument identifies
//! which.
//!
//! **Penalties**:
//! - Imprisonment up to 5 YEARS (one of the longest in the Code).
//! - Fine up to $100,000 individual / $500,000 corporation under
//!   § 7201 itself; 18 U.S.C. § 3571 (Criminal Fines Improvement
//!   Act) supersedes to $250,000 individual / $500,000
//!   corporation.
//! - BOTH imprisonment and fine permitted.
//! - Costs of prosecution.
//!
//! **Cheek defense — Cheek v. United States, 498 U.S. 192 (1991)**.
//! Good-faith subjective misunderstanding of the law negates
//! willfulness element. NOT a defense for disagreement with the
//! law (constitutional challenges, tax-protester arguments).
//! Same Cheek defense available as for § 7206.
//!
//! **§ 6531 criminal statute of limitations** — 6 YEARS for §
//! 7201 (one of several 6-year tax crimes).
//!
//! **Parallel civil consequences**. Conviction under § 7201
//! triggers § 6663 civil fraud 75% penalty + § 6501(c)(1)
//! UNLIMITED ASED + § 6501(c)(2) UNLIMITED ASED for willful
//! evasion. Spies-Daly doctrine permits parallel civil + criminal.
//! Double jeopardy does NOT bar civil penalty after criminal
//! conviction (or acquittal).
//!
//! Citations: IRC § 7201 attempt to evade tax; § 6531 criminal
//! statute of limitations (6 years for § 7201); 18 U.S.C. § 3571
//! Criminal Fines Improvement Act; Spies v. United States, 317
//! U.S. 492 (1943) affirmative-act doctrine; Sansone v. United
//! States, 380 U.S. 343 (1965) evasion-of-assessment vs
//! evasion-of-payment; Cheek v. United States, 498 U.S. 192
//! (1991) subjective good-faith defense; IRM 9.1.3 Criminal
//! Statutory Provisions and Common Law; § 6663 (civil fraud
//! parallel 75%); § 6501(c)(1) / (c)(2) unlimited ASED;
//! § 7206 (related fraud and false statements 3-year felony).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    Individual,
    Corporation,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvasionForm {
    /// Evasion of assessment — affirmative act preventing IRS
    /// from determining correct tax.
    EvasionOfAssessment,
    /// Evasion of payment — affirmative act preventing IRS from
    /// collecting tax already assessed.
    EvasionOfPayment,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section7201Input {
    pub entity_type: EntityType,
    pub evasion_form: EvasionForm,
    // Four-element test:
    /// Element 1 — Existence of tax deficiency. Amount in cents.
    pub tax_deficiency_amount_cents: i64,
    /// Element 2 — Willfulness (voluntary intentional violation
    /// of known duty).
    pub willfulness_voluntary_intentional: bool,
    /// Element 3 — Affirmative act of evasion (any Spies
    /// indicium or other affirmative conduct).
    pub affirmative_act_of_evasion: bool,
    /// Element 4 — Substantial amount of tax owed (case law:
    /// no fixed threshold but materially significant).
    pub substantial_amount: bool,

    // Spies affirmative-act indicia (Spies v. United States,
    // 317 U.S. 492 (1943)):
    pub spies_double_set_of_books: bool,
    pub spies_false_entries_or_alterations: bool,
    pub spies_false_invoices_or_documents: bool,
    pub spies_destruction_of_books_or_records: bool,
    pub spies_concealment_of_assets: bool,
    pub spies_covering_up_sources_of_income: bool,
    pub spies_handling_affairs_to_avoid_records: bool,

    /// Whether the only conduct alleged is FAILURE TO FILE /
    /// FAILURE TO PAY (mere omission — does NOT satisfy
    /// affirmative-act element).
    pub conduct_is_mere_omission: bool,

    /// Whether the Cheek defense is asserted (good-faith
    /// misunderstanding of law).
    pub cheek_defense_asserted: bool,
    /// Whether the Cheek defense is successful.
    pub cheek_defense_successful: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section7201Result {
    pub felony_prosecution_authorized: bool,
    pub maximum_imprisonment_years: u32,
    pub maximum_fine_cents: i64,
    pub four_elements_satisfied_count: u32,
    pub spies_indicia_count: u32,
    pub mere_omission_defeats_affirmative_act: bool,
    pub cheek_defense_engaged_and_successful: bool,
    pub criminal_sol_years: u32,
    pub section_6663_civil_parallel_available: bool,
    /// Whether § 6501(c)(1) unlimited ASED triggered.
    pub unlimited_ased_triggered: bool,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section7201Input) -> Section7201Result {
    let mut notes: Vec<String> = Vec::new();

    let max_fine = match input.entity_type {
        EntityType::Individual => 25_000_000i64,
        EntityType::Corporation => 50_000_000i64,
    };

    let spies_count = count_spies_indicia(input);

    notes.push(
        "§ 7201 — apex criminal tax felony; up to 5 YEARS imprisonment + fine + costs of prosecution; pairs with § 7206 (3-year felony) and § 6663 (civil fraud 75%)"
            .to_string(),
    );

    notes.push(
        "18 U.S.C. § 3571 (Criminal Fines Improvement Act) — maximum fines for § 7201 violations: $250,000 individual / $500,000 corporation; supersedes § 7201's original $100,000 individual cap"
            .to_string(),
    );

    notes.push(
        "§ 6531 criminal statute of limitations — 6 years for § 7201 attempt to evade".to_string(),
    );

    notes.push(format!(
        "Spies v. United States, 317 U.S. 492 (1943) — {} of 7 enumerated affirmative-act indicia present (double books / false entries / false invoices / destruction of records / concealment of assets / covering up income sources / avoiding usual records)",
        spies_count
    ));

    if input.conduct_is_mere_omission {
        notes.push(
            "§ 7201 — mere failure to file or failure to pay (omission) does NOT constitute attempt to evade; Spies affirmative-act doctrine requires conduct 'the likely effect of which would be to mislead or to conceal'"
                .to_string(),
        );
    }

    let cheek_engaged = input.cheek_defense_asserted && input.cheek_defense_successful;
    if input.cheek_defense_asserted {
        notes.push(
            "Cheek v. United States, 498 U.S. 192 (1991) — good-faith subjective misunderstanding of law negates willfulness; NOT a defense for disagreement with the law (constitutional challenges, tax-protester arguments)"
                .to_string(),
        );
        if cheek_engaged {
            notes.push(
                "Cheek defense SUCCESSFUL — subjective good-faith belief established; willfulness element of § 7201 defeated"
                    .to_string(),
            );
        }
    }

    match input.evasion_form {
        EvasionForm::EvasionOfAssessment => {
            notes.push(
                "Sansone v. United States, 380 U.S. 343 (1965) — evasion of ASSESSMENT form: affirmative act prevents IRS from determining correct tax (false return, hidden income)"
                    .to_string(),
            );
        }
        EvasionForm::EvasionOfPayment => {
            notes.push(
                "Sansone v. United States, 380 U.S. 343 (1965) — evasion of PAYMENT form: affirmative act AFTER assessment prevents IRS from collecting (concealment of assets, transfers to nominees)"
                    .to_string(),
            );
        }
    }

    let element_1_deficiency_exists = input.tax_deficiency_amount_cents > 0;
    let element_2_willful = input.willfulness_voluntary_intentional;
    let element_3_affirmative = input.affirmative_act_of_evasion && !input.conduct_is_mere_omission;
    let element_4_substantial = input.substantial_amount;

    let count = [
        element_1_deficiency_exists,
        element_2_willful,
        element_3_affirmative,
        element_4_substantial,
    ]
    .iter()
    .filter(|&&b| b)
    .count() as u32;

    let all_four = count == 4;
    let felony_authorized = all_four && !cheek_engaged;

    notes.push(format!(
        "§ 7201 four-element test: {}/4 satisfied — (1) tax deficiency + (2) willfulness + (3) affirmative act of evasion + (4) substantial amount",
        count
    ));

    if felony_authorized {
        notes.push(
            "§ 7201 — all four elements satisfied; FELONY PROSECUTION AUTHORIZED".to_string(),
        );
    } else if !all_four {
        notes.push(
            "§ 7201 — government has not established all four elements beyond reasonable doubt; prosecution NOT authorized"
                .to_string(),
        );
    } else if cheek_engaged {
        notes.push(
            "§ 7201 — Cheek defense defeats willfulness element; prosecution NOT authorized despite other elements satisfied"
                .to_string(),
        );
    }

    notes.push(
        "Spies-Daly doctrine — § 7201 criminal prosecution may PROCEED IN PARALLEL with § 6663 civil fraud 75% penalty; double jeopardy does NOT bar civil penalty after criminal conviction (or acquittal)"
            .to_string(),
    );

    notes.push(
        "§ 6501(c)(1) UNLIMITED ASED — fraud established under § 7201 triggers unlimited assessment statute of limitations; § 6501(c)(2) UNLIMITED ASED for willful attempt to evade tax"
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

    Section7201Result {
        felony_prosecution_authorized: felony_authorized,
        maximum_imprisonment_years: 5,
        maximum_fine_cents: max_fine,
        four_elements_satisfied_count: count,
        spies_indicia_count: spies_count,
        mere_omission_defeats_affirmative_act: input.conduct_is_mere_omission,
        cheek_defense_engaged_and_successful: cheek_engaged,
        criminal_sol_years: 6,
        section_6663_civil_parallel_available: true,
        unlimited_ased_triggered: felony_authorized,
        citation: "IRC §§ 7201, 6531, 6663, 6501(c)(1), 6501(c)(2), 7206; 18 U.S.C. § 3571 (Criminal Fines Improvement Act); Spies v. United States, 317 U.S. 492 (1943); Sansone v. United States, 380 U.S. 343 (1965); Cheek v. United States, 498 U.S. 192 (1991); IRM 9.1.3",
        notes,
    }
}

fn count_spies_indicia(i: &Section7201Input) -> u32 {
    [
        i.spies_double_set_of_books,
        i.spies_false_entries_or_alterations,
        i.spies_false_invoices_or_documents,
        i.spies_destruction_of_books_or_records,
        i.spies_concealment_of_assets,
        i.spies_covering_up_sources_of_income,
        i.spies_handling_affairs_to_avoid_records,
    ]
    .iter()
    .filter(|&&b| b)
    .count() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full_prosecution() -> Section7201Input {
        Section7201Input {
            entity_type: EntityType::Individual,
            evasion_form: EvasionForm::EvasionOfAssessment,
            tax_deficiency_amount_cents: 1_000_000_00,
            willfulness_voluntary_intentional: true,
            affirmative_act_of_evasion: true,
            substantial_amount: true,
            spies_double_set_of_books: false,
            spies_false_entries_or_alterations: false,
            spies_false_invoices_or_documents: false,
            spies_destruction_of_books_or_records: false,
            spies_concealment_of_assets: false,
            spies_covering_up_sources_of_income: false,
            spies_handling_affairs_to_avoid_records: false,
            conduct_is_mere_omission: false,
            cheek_defense_asserted: false,
            cheek_defense_successful: false,
        }
    }

    #[test]
    fn full_four_elements_authorizes_felony() {
        let r = check(&full_prosecution());
        assert!(r.felony_prosecution_authorized);
        assert_eq!(r.four_elements_satisfied_count, 4);
        assert_eq!(r.maximum_imprisonment_years, 5);
        assert!(r.unlimited_ased_triggered);
    }

    #[test]
    fn maximum_imprisonment_5_years() {
        let r = check(&full_prosecution());
        assert_eq!(r.maximum_imprisonment_years, 5);
    }

    #[test]
    fn individual_max_fine_250k() {
        let r = check(&full_prosecution());
        assert_eq!(r.maximum_fine_cents, 25_000_000);
    }

    #[test]
    fn corporation_max_fine_500k() {
        let mut i = full_prosecution();
        i.entity_type = EntityType::Corporation;
        let r = check(&i);
        assert_eq!(r.maximum_fine_cents, 50_000_000);
    }

    #[test]
    fn missing_deficiency_defeats_prosecution() {
        let mut i = full_prosecution();
        i.tax_deficiency_amount_cents = 0;
        let r = check(&i);
        assert!(!r.felony_prosecution_authorized);
        assert_eq!(r.four_elements_satisfied_count, 3);
    }

    #[test]
    fn missing_willfulness_defeats_prosecution() {
        let mut i = full_prosecution();
        i.willfulness_voluntary_intentional = false;
        let r = check(&i);
        assert!(!r.felony_prosecution_authorized);
    }

    #[test]
    fn missing_affirmative_act_defeats_prosecution() {
        let mut i = full_prosecution();
        i.affirmative_act_of_evasion = false;
        let r = check(&i);
        assert!(!r.felony_prosecution_authorized);
    }

    #[test]
    fn missing_substantial_amount_defeats_prosecution() {
        let mut i = full_prosecution();
        i.substantial_amount = false;
        let r = check(&i);
        assert!(!r.felony_prosecution_authorized);
    }

    #[test]
    fn mere_omission_defeats_affirmative_act() {
        let mut i = full_prosecution();
        i.conduct_is_mere_omission = true;
        let r = check(&i);
        assert!(r.mere_omission_defeats_affirmative_act);
        assert!(!r.felony_prosecution_authorized);
        assert!(r.notes.iter().any(|n| n.contains("§ 7201")
            && n.contains("mere failure to file")
            && n.contains("Spies")));
    }

    #[test]
    fn spies_indicia_count_zero_when_none_present() {
        let r = check(&full_prosecution());
        assert_eq!(r.spies_indicia_count, 0);
    }

    #[test]
    fn all_seven_spies_indicia_count() {
        let mut i = full_prosecution();
        i.spies_double_set_of_books = true;
        i.spies_false_entries_or_alterations = true;
        i.spies_false_invoices_or_documents = true;
        i.spies_destruction_of_books_or_records = true;
        i.spies_concealment_of_assets = true;
        i.spies_covering_up_sources_of_income = true;
        i.spies_handling_affairs_to_avoid_records = true;
        let r = check(&i);
        assert_eq!(r.spies_indicia_count, 7);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Spies v. United States") && n.contains("7 of 7")));
    }

    #[test]
    fn cheek_defense_successful_defeats_prosecution() {
        let mut i = full_prosecution();
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
        let mut i = full_prosecution();
        i.cheek_defense_asserted = true;
        i.cheek_defense_successful = false;
        let r = check(&i);
        assert!(r.felony_prosecution_authorized);
        assert!(!r.cheek_defense_engaged_and_successful);
    }

    #[test]
    fn cheek_case_note_describes_subjective_belief() {
        let mut i = full_prosecution();
        i.cheek_defense_asserted = true;
        let r = check(&i);
        assert!(r.notes.iter().any(
            |n| n.contains("Cheek v. United States, 498 U.S. 192 (1991)")
                && n.contains("subjective")
        ));
    }

    #[test]
    fn sansone_evasion_of_assessment_note_present() {
        let r = check(&full_prosecution());
        assert!(r.notes.iter().any(|n| n
            .contains("Sansone v. United States, 380 U.S. 343 (1965)")
            && n.contains("evasion of ASSESSMENT")));
    }

    #[test]
    fn sansone_evasion_of_payment_note_when_payment_form() {
        let mut i = full_prosecution();
        i.evasion_form = EvasionForm::EvasionOfPayment;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("evasion of PAYMENT")));
    }

    #[test]
    fn four_element_truth_table() {
        for d in [false, true] {
            for w in [false, true] {
                for a in [false, true] {
                    for s in [false, true] {
                        let mut i = full_prosecution();
                        i.tax_deficiency_amount_cents = if d { 1_000_000 } else { 0 };
                        i.willfulness_voluntary_intentional = w;
                        i.affirmative_act_of_evasion = a;
                        i.substantial_amount = s;
                        let r = check(&i);
                        let all_four = d && w && a && s;
                        assert_eq!(r.felony_prosecution_authorized, all_four);
                    }
                }
            }
        }
    }

    #[test]
    fn criminal_sol_6_years() {
        let r = check(&full_prosecution());
        assert_eq!(r.criminal_sol_years, 6);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6531") && n.contains("6 years for § 7201")));
    }

    #[test]
    fn section_6663_civil_parallel_always_available() {
        let r = check(&full_prosecution());
        assert!(r.section_6663_civil_parallel_available);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Spies-Daly doctrine") && n.contains("PARALLEL")));
    }

    #[test]
    fn unlimited_ased_triggered_when_prosecution_authorized() {
        let r = check(&full_prosecution());
        assert!(r.unlimited_ased_triggered);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6501(c)(1) UNLIMITED ASED")));
    }

    #[test]
    fn unlimited_ased_not_triggered_when_prosecution_not_authorized() {
        let mut i = full_prosecution();
        i.willfulness_voluntary_intentional = false;
        let r = check(&i);
        assert!(!r.unlimited_ased_triggered);
    }

    #[test]
    fn section_7491_burden_shifts_excluded_note() {
        let r = check(&full_prosecution());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 7491") && n.contains("BEYOND REASONABLE DOUBT")));
    }

    #[test]
    fn cfia_18_usc_3571_supersedes_note() {
        let r = check(&full_prosecution());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("18 U.S.C. § 3571") && n.contains("$250,000")));
    }

    #[test]
    fn irm_9_1_3_note_present() {
        let r = check(&full_prosecution());
        assert!(r.notes.iter().any(|n| n.contains("IRM 9.1.3")));
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&full_prosecution());
        assert!(r.citation.contains("§§ 7201, 6531, 6663"));
        assert!(r.citation.contains("6501(c)(1)"));
        assert!(r.citation.contains("6501(c)(2)"));
        assert!(r.citation.contains(", 7206"));
        assert!(r.citation.contains("18 U.S.C. § 3571"));
        assert!(r.citation.contains("Spies v. United States"));
        assert!(r.citation.contains("Sansone v. United States"));
        assert!(r.citation.contains("Cheek v. United States"));
        assert!(r.citation.contains("IRM 9.1.3"));
    }

    #[test]
    fn affirmative_act_with_omission_flag_still_defeats() {
        let mut i = full_prosecution();
        i.affirmative_act_of_evasion = true;
        i.conduct_is_mere_omission = true;
        let r = check(&i);
        assert!(!r.felony_prosecution_authorized);
    }

    #[test]
    fn cheek_only_defeats_when_successful_and_asserted() {
        for asserted in [false, true] {
            for successful in [false, true] {
                let mut i = full_prosecution();
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
    fn negative_deficiency_treated_as_no_deficiency() {
        let mut i = full_prosecution();
        i.tax_deficiency_amount_cents = -10_000;
        let r = check(&i);
        assert!(!r.felony_prosecution_authorized);
    }

    #[test]
    fn spies_indicia_truth_table_count() {
        let mut i = full_prosecution();
        assert_eq!(check(&i).spies_indicia_count, 0);

        i.spies_double_set_of_books = true;
        assert_eq!(check(&i).spies_indicia_count, 1);

        i.spies_false_entries_or_alterations = true;
        i.spies_false_invoices_or_documents = true;
        i.spies_destruction_of_books_or_records = true;
        assert_eq!(check(&i).spies_indicia_count, 4);

        i.spies_concealment_of_assets = true;
        i.spies_covering_up_sources_of_income = true;
        i.spies_handling_affairs_to_avoid_records = true;
        assert_eq!(check(&i).spies_indicia_count, 7);
    }

    #[test]
    fn pairs_with_7206_citation_reference() {
        let r = check(&full_prosecution());
        assert!(r.citation.contains(", 7206"));
    }

    #[test]
    fn four_element_count_increments_per_element() {
        let mut i = full_prosecution();
        i.tax_deficiency_amount_cents = 0;
        i.willfulness_voluntary_intentional = false;
        i.affirmative_act_of_evasion = false;
        i.substantial_amount = false;
        assert_eq!(check(&i).four_elements_satisfied_count, 0);

        i.tax_deficiency_amount_cents = 100_000;
        assert_eq!(check(&i).four_elements_satisfied_count, 1);

        i.willfulness_voluntary_intentional = true;
        assert_eq!(check(&i).four_elements_satisfied_count, 2);

        i.affirmative_act_of_evasion = true;
        assert_eq!(check(&i).four_elements_satisfied_count, 3);

        i.substantial_amount = true;
        assert_eq!(check(&i).four_elements_satisfied_count, 4);
    }
}
