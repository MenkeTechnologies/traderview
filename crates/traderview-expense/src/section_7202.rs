//! IRC § 7202 — Willful failure to collect or pay over tax.
//! Criminal FELONY (5-year imprisonment cap) targeting
//! employers and other responsible persons who fail to truthfully
//! account for and pay over TRUST FUND TAXES (federal income tax
//! withheld from employee wages + employee FICA share). Criminal
//! counterpart to `section_6672` Trust Fund Recovery Penalty
//! (civil 100% penalty). Pairs with `section_7201` (5-year felony
//! / attempt to evade), `section_7206` (3-year felony / fraud and
//! false statements / tax perjury), and `section_7203` (1-year
//! misdemeanor / willful failure to file/pay).
//!
//! **Critical distinction between § 7202 and § 6672**: same
//! conduct (willful failure to pay over trust fund taxes by a
//! responsible person) gives rise to BOTH a § 6672 100% civil
//! penalty AND a § 7202 5-year felony. Spies-Daly doctrine
//! permits parallel civil + criminal. Trader-landlord-business
//! operational concern in any entity with W-2 employees — LLC
//! with employees, S-corp with shareholder-employees, C-corp.
//!
//! **§ 7202 four-element test** (government bears BEYOND
//! REASONABLE DOUBT burden):
//!
//! 1. DUTY to collect, account for, or pay over tax (typically
//!    payroll taxes).
//! 2. WILLFUL failure to do so (voluntary intentional violation
//!    of known legal duty; no bad/evil intent required).
//! 3. AMOUNT REQUIRED to be withheld and paid (substantial
//!    amount).
//! 4. Defendant was a RESPONSIBLE PERSON.
//!
//! All four required; failure of any defeats prosecution.
//!
//! **Trust fund taxes reached by § 7202**:
//!
//! - Federal income tax WITHHELD from employee wages (§ 3402)
//! - Employee FICA share (§ 3101) — Social Security + Medicare
//! - Federal Unemployment Tax (FUTA, § 3301)
//!
//! NOT REACHED by § 7202 (separate civil + criminal frameworks):
//! - Employer FICA match (§ 3111) — not trust fund
//! - Sales and excise taxes
//!
//! **Responsible person standard** — same standard as § 6672:
//! status, duty, AND authority to avoid the employer's default
//! in collection or payment of taxes. Criminal liability depends
//! on the person's duties, position, involvement in company
//! operations, and extent of control over financial resources.
//! Includes officers, directors, members, shareholders, payroll
//! managers with check-signing authority.
//!
//! **Penalties**:
//! - Imprisonment up to 5 YEARS
//! - Fine up to $10,000 under § 7202's original text
//! - 18 U.S.C. § 3571 (Criminal Fines Improvement Act)
//!   supersedes to $250,000 individual / $500,000 corporation
//! - BOTH imprisonment and fine permitted
//! - Costs of prosecution
//!
//! **Willfulness standard** — same as Title 26 generally:
//! voluntary, intentional violation of a known legal duty. NO
//! requirement to prove evil or bad intent. An act founded in
//! good intent that voluntarily and intentionally violates a
//! known legal duty supports willfulness. Same Cheek defense
//! available — Cheek v. United States, 498 U.S. 192 (1991)
//! good-faith subjective belief negates willfulness, EVEN IF
//! OBJECTIVELY UNREASONABLE.
//!
//! **§ 6531 criminal SOL** — 6 years for § 7202.
//!
//! **Parallel civil consequences**:
//! - § 6672 Trust Fund Recovery Penalty (100% of unpaid trust
//!   fund taxes) routinely imposed alongside § 7202 prosecution
//! - § 6651(a)(2) civil failure-to-pay penalty (0.5% per month)
//! - 11 U.S.C. § 523(a)(7) — § 6672 TFRP NONDISCHARGEABLE in
//!   personal bankruptcy
//! - § 7491 burden shifts do NOT apply to criminal prosecutions
//!   (BEYOND REASONABLE DOUBT burden)
//!
//! Citations: IRC § 7202 willful failure to collect or pay over
//! tax; § 6672 Trust Fund Recovery Penalty (civil counterpart);
//! § 6531 criminal SOL (6 years for § 7202); 18 U.S.C. § 3571
//! Criminal Fines Improvement Act; § 3402 income tax withholding;
//! § 3101 employee FICA; § 3301 FUTA; § 3111 employer FICA
//! (excluded); Cheek v. United States, 498 U.S. 192 (1991)
//! subjective good-faith defense; § 6651(a)(2) civil failure-to-
//! pay penalty; 11 U.S.C. § 523(a)(7) nondischargeable in
//! bankruptcy; § 7491 burden shifts excluded for criminal cases;
//! IRM 9.1.3; IRM 8.25.1 (TFRP procedural manual).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    Individual,
    Corporation,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section7202Input {
    pub entity_type: EntityType,
    // Four-element test:
    /// Element 1 — duty to collect, account for, or pay over
    /// tax (typically payroll trust fund taxes).
    pub element_1_duty_to_collect_account_or_pay_over: bool,
    /// Element 2 — willful failure to perform the duty
    /// (voluntary intentional violation of known legal duty).
    pub element_2_willful_failure: bool,
    /// Element 3 — amount required to be withheld and paid
    /// (substantial amount).
    pub element_3_amount_required_to_be_withheld: bool,
    /// Element 4 — defendant was a RESPONSIBLE PERSON (status,
    /// duty, AND authority to avoid default).
    pub element_4_responsible_person: bool,
    /// Whether the Cheek defense (good-faith subjective belief)
    /// is asserted.
    pub cheek_defense_asserted: bool,
    /// Whether the Cheek defense is successful.
    pub cheek_defense_successful: bool,
    /// Whether the § 6672 TFRP (civil 100%) has also been imposed
    /// on the same conduct (parallel civil-criminal track).
    pub parallel_section_6672_tfrp_imposed: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section7202Result {
    pub felony_prosecution_authorized: bool,
    pub maximum_imprisonment_years: u32,
    pub maximum_fine_cents: i64,
    pub four_elements_satisfied_count: u32,
    pub responsible_person_status_engaged: bool,
    pub cheek_defense_engaged_and_successful: bool,
    pub criminal_sol_years: u32,
    pub parallel_section_6672_tfrp_available: bool,
    pub parallel_section_6651_a_2_civil_failure_to_pay_available: bool,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section7202Input) -> Section7202Result {
    let mut notes: Vec<String> = Vec::new();

    let max_fine = match input.entity_type {
        EntityType::Individual => 25_000_000i64,
        EntityType::Corporation => 50_000_000i64,
    };

    notes.push(
        "§ 7202 — criminal FELONY (5-year imprisonment cap); criminal counterpart to § 6672 Trust Fund Recovery Penalty (civil 100%); same conduct triggers BOTH § 7202 felony and § 6672 100% civil penalty"
            .to_string(),
    );

    notes.push(
        "18 U.S.C. § 3571 (Criminal Fines Improvement Act) — maximum fines for § 7202 violations: $250,000 individual / $500,000 corporation; supersedes § 7202's original $10,000 cap"
            .to_string(),
    );

    notes.push(
        "§ 6531 criminal statute of limitations — 6 years for § 7202 willful failure to collect or pay over tax"
            .to_string(),
    );

    notes.push(
        "trust fund taxes reached by § 7202: § 3402 federal income tax withheld from employee wages + § 3101 employee FICA share (Social Security + Medicare) + § 3301 FUTA; NOT REACHED: § 3111 employer FICA match (separate framework)"
            .to_string(),
    );

    notes.push(
        "§ 7202 responsible person standard (same as § 6672) — status, duty, AND authority to avoid employer's default in collection or payment of taxes; criminal liability depends on duties, position, involvement in operations, extent of control over financial resources; reaches officers + directors + members + shareholders + payroll managers with check-signing authority"
            .to_string(),
    );

    let cheek_engaged = input.cheek_defense_asserted && input.cheek_defense_successful;
    if input.cheek_defense_asserted {
        notes.push(
            "Cheek v. United States, 498 U.S. 192 (1991) — genuine good-faith subjective belief negates willfulness EVEN IF OBJECTIVELY UNREASONABLE; subjective belief test; NOT a defense for disagreement with the law"
                .to_string(),
        );
        if cheek_engaged {
            notes.push(
                "Cheek defense SUCCESSFUL — subjective good-faith belief established; willfulness element of § 7202 defeated"
                    .to_string(),
            );
        }
    }

    let count = [
        input.element_1_duty_to_collect_account_or_pay_over,
        input.element_2_willful_failure,
        input.element_3_amount_required_to_be_withheld,
        input.element_4_responsible_person,
    ]
    .iter()
    .filter(|&&b| b)
    .count() as u32;

    let all_four = count == 4;
    let felony_authorized = all_four && !cheek_engaged;

    notes.push(format!(
        "§ 7202 four-element test: {}/4 satisfied — (1) duty to collect / account for / pay over tax + (2) willful failure + (3) amount required to be withheld and paid + (4) defendant was a responsible person",
        count
    ));

    if felony_authorized {
        notes.push(
            "§ 7202 — all four elements satisfied; FELONY PROSECUTION AUTHORIZED".to_string(),
        );
    } else if !all_four {
        notes.push(
            "§ 7202 — government has not established all four elements beyond reasonable doubt; prosecution NOT authorized"
                .to_string(),
        );
    } else if cheek_engaged {
        notes.push(
            "§ 7202 — Cheek defense defeats willfulness element; prosecution NOT authorized despite other elements satisfied"
                .to_string(),
        );
    }

    notes.push(
        "willfulness standard under § 7202 (same as all Title 26 offenses) — voluntary intentional violation of known legal duty; NO requirement to prove evil or bad intent; act founded in good intent that voluntarily violates known duty supports willfulness"
            .to_string(),
    );

    notes.push(
        "Spies-Daly doctrine — § 7202 prosecution may PROCEED IN PARALLEL with § 6672 Trust Fund Recovery Penalty (100% civil) + § 6651(a)(2) failure-to-pay (0.5% per month); double jeopardy does NOT bar civil after criminal"
            .to_string(),
    );

    if input.parallel_section_6672_tfrp_imposed {
        notes.push(
            "§ 6672 TFRP already imposed — parallel civil track engaged; 11 U.S.C. § 523(a)(7) NONDISCHARGEABLE in personal bankruptcy"
                .to_string(),
        );
    }

    notes.push(
        "§ 7491 burden of proof shifts do NOT apply to criminal prosecutions; government bears BEYOND REASONABLE DOUBT burden on each element"
            .to_string(),
    );

    notes.push(
        "IRM 9.1.3 — Criminal Statutory Provisions and Common Law procedural manual governs IRS Criminal Investigation (CI) referral process to DOJ Tax Division; IRM 8.25.1 — TFRP procedural manual for parallel civil track"
            .to_string(),
    );

    Section7202Result {
        felony_prosecution_authorized: felony_authorized,
        maximum_imprisonment_years: 5,
        maximum_fine_cents: max_fine,
        four_elements_satisfied_count: count,
        responsible_person_status_engaged: input.element_4_responsible_person,
        cheek_defense_engaged_and_successful: cheek_engaged,
        criminal_sol_years: 6,
        parallel_section_6672_tfrp_available: true,
        parallel_section_6651_a_2_civil_failure_to_pay_available: true,
        citation: "IRC §§ 7202, 6672, 6531, 6651(a)(2), 3402, 3101, 3301, 3111; 18 U.S.C. § 3571 (Criminal Fines Improvement Act); 11 U.S.C. § 523(a)(7); Cheek v. United States, 498 U.S. 192 (1991); IRM 9.1.3; IRM 8.25.1",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full_prosecution() -> Section7202Input {
        Section7202Input {
            entity_type: EntityType::Individual,
            element_1_duty_to_collect_account_or_pay_over: true,
            element_2_willful_failure: true,
            element_3_amount_required_to_be_withheld: true,
            element_4_responsible_person: true,
            cheek_defense_asserted: false,
            cheek_defense_successful: false,
            parallel_section_6672_tfrp_imposed: false,
        }
    }

    #[test]
    fn full_four_elements_authorizes_felony() {
        let r = check(&full_prosecution());
        assert!(r.felony_prosecution_authorized);
        assert_eq!(r.four_elements_satisfied_count, 4);
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
    fn missing_duty_defeats_prosecution() {
        let mut i = full_prosecution();
        i.element_1_duty_to_collect_account_or_pay_over = false;
        let r = check(&i);
        assert!(!r.felony_prosecution_authorized);
        assert_eq!(r.four_elements_satisfied_count, 3);
    }

    #[test]
    fn missing_willfulness_defeats_prosecution() {
        let mut i = full_prosecution();
        i.element_2_willful_failure = false;
        let r = check(&i);
        assert!(!r.felony_prosecution_authorized);
    }

    #[test]
    fn missing_amount_required_defeats_prosecution() {
        let mut i = full_prosecution();
        i.element_3_amount_required_to_be_withheld = false;
        let r = check(&i);
        assert!(!r.felony_prosecution_authorized);
    }

    #[test]
    fn missing_responsible_person_defeats_prosecution() {
        let mut i = full_prosecution();
        i.element_4_responsible_person = false;
        let r = check(&i);
        assert!(!r.felony_prosecution_authorized);
        assert!(!r.responsible_person_status_engaged);
    }

    #[test]
    fn four_element_truth_table() {
        for e1 in [false, true] {
            for e2 in [false, true] {
                for e3 in [false, true] {
                    for e4 in [false, true] {
                        let mut i = full_prosecution();
                        i.element_1_duty_to_collect_account_or_pay_over = e1;
                        i.element_2_willful_failure = e2;
                        i.element_3_amount_required_to_be_withheld = e3;
                        i.element_4_responsible_person = e4;
                        let r = check(&i);
                        let all_four = e1 && e2 && e3 && e4;
                        assert_eq!(r.felony_prosecution_authorized, all_four);
                    }
                }
            }
        }
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
    fn cheek_case_objectively_unreasonable_belief_note() {
        let mut i = full_prosecution();
        i.cheek_defense_asserted = true;
        let r = check(&i);
        assert!(r.notes.iter().any(
            |n| n.contains("Cheek v. United States") && n.contains("OBJECTIVELY UNREASONABLE")
        ));
    }

    #[test]
    fn willfulness_no_evil_intent_required_note() {
        let r = check(&full_prosecution());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("willfulness standard")
                && n.contains("NO requirement to prove evil")));
    }

    #[test]
    fn trust_fund_taxes_note_lists_three_categories() {
        let r = check(&full_prosecution());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 3402") && n.contains("§ 3101") && n.contains("§ 3301")));
    }

    #[test]
    fn employer_fica_match_excluded_note() {
        let r = check(&full_prosecution());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 3111 employer FICA match") && n.contains("NOT REACHED")));
    }

    #[test]
    fn responsible_person_standard_same_as_6672_note() {
        let r = check(&full_prosecution());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 7202 responsible person standard")
                && n.contains("same as § 6672")));
    }

    #[test]
    fn criminal_sol_6_years() {
        let r = check(&full_prosecution());
        assert_eq!(r.criminal_sol_years, 6);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6531") && n.contains("6 years for § 7202")));
    }

    #[test]
    fn parallel_6672_tfrp_always_available() {
        let r = check(&full_prosecution());
        assert!(r.parallel_section_6672_tfrp_available);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Spies-Daly doctrine") && n.contains("§ 6672")));
    }

    #[test]
    fn parallel_6672_tfrp_imposed_engages_nondischargeable_note() {
        let mut i = full_prosecution();
        i.parallel_section_6672_tfrp_imposed = true;
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6672 TFRP already imposed")
                && n.contains("§ 523(a)(7) NONDISCHARGEABLE")));
    }

    #[test]
    fn no_parallel_tfrp_no_nondischargeable_note() {
        let r = check(&full_prosecution());
        assert!(!r
            .notes
            .iter()
            .any(|n| n.contains("§ 6672 TFRP already imposed")));
    }

    #[test]
    fn parallel_6651_a_2_failure_to_pay_always_available() {
        let r = check(&full_prosecution());
        assert!(r.parallel_section_6651_a_2_civil_failure_to_pay_available);
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
    fn irm_8_25_1_tfrp_note_present() {
        let r = check(&full_prosecution());
        assert!(r.notes.iter().any(|n| n.contains("IRM 8.25.1")));
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&full_prosecution());
        assert!(r.citation.contains("§§ 7202, 6672, 6531"));
        assert!(r.citation.contains("6651(a)(2)"));
        assert!(r.citation.contains("3402"));
        assert!(r.citation.contains("3101"));
        assert!(r.citation.contains("3301"));
        assert!(r.citation.contains("3111"));
        assert!(r.citation.contains("18 U.S.C. § 3571"));
        assert!(r.citation.contains("11 U.S.C. § 523(a)(7)"));
        assert!(r.citation.contains("Cheek v. United States"));
        assert!(r.citation.contains("IRM 9.1.3"));
        assert!(r.citation.contains("IRM 8.25.1"));
    }

    #[test]
    fn distinction_from_6672_note_describes_parallel() {
        let r = check(&full_prosecution());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 7202 — criminal FELONY")
                && n.contains("§ 6672")
                && n.contains("BOTH § 7202 felony and § 6672")));
    }

    #[test]
    fn four_element_count_increments_per_element() {
        let mut i = full_prosecution();
        i.element_1_duty_to_collect_account_or_pay_over = false;
        i.element_2_willful_failure = false;
        i.element_3_amount_required_to_be_withheld = false;
        i.element_4_responsible_person = false;
        assert_eq!(check(&i).four_elements_satisfied_count, 0);

        i.element_1_duty_to_collect_account_or_pay_over = true;
        assert_eq!(check(&i).four_elements_satisfied_count, 1);

        i.element_2_willful_failure = true;
        assert_eq!(check(&i).four_elements_satisfied_count, 2);

        i.element_3_amount_required_to_be_withheld = true;
        assert_eq!(check(&i).four_elements_satisfied_count, 3);

        i.element_4_responsible_person = true;
        assert_eq!(check(&i).four_elements_satisfied_count, 4);
    }

    #[test]
    fn responsible_person_status_engages_with_element_4() {
        let r = check(&full_prosecution());
        assert!(r.responsible_person_status_engaged);

        let mut i = full_prosecution();
        i.element_4_responsible_person = false;
        assert!(!check(&i).responsible_person_status_engaged);
    }

    #[test]
    fn pairs_with_6672_civil_counterpart() {
        let r = check(&full_prosecution());
        assert!(r.citation.contains("6672"));
        assert!(r.notes.iter().any(|n| n.contains("§ 6672")));
    }
}
