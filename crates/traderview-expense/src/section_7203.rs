//! IRC § 7203 — Willful failure to file return, supply
//! information, or pay tax. Criminal MISDEMEANOR (1-year
//! imprisonment cap) — distinct from `section_7201` 5-year
//! felony (attempt to evade) and `section_7206` 3-year felony
//! (fraud and false statements / tax perjury). Together § 7201,
//! § 7203, and § 7206 form the principal criminal tax statute
//! trio.
//!
//! The critical distinction between § 7203 and § 7201 is the
//! Spies doctrine — § 7203 reaches MERE OMISSIONS (failure to
//! file, pay, or supply information), while § 7201 requires
//! AFFIRMATIVE ACTS of evasion. A willful failure to file alone
//! is § 7203 misdemeanor; the same failure coupled with
//! affirmative acts of concealment elevates to § 7201 felony.
//!
//! **§ 7203 three-element test** (government bears BEYOND
//! REASONABLE DOUBT burden):
//!
//! 1. Person was REQUIRED BY LAW to file return, pay tax,
//!    supply information, or keep records.
//! 2. FAILURE to do so at the time required by law.
//! 3. WILLFULNESS — voluntary intentional violation of a known
//!    legal duty.
//!
//! All three required; failure of any defeats prosecution.
//!
//! **§ 7203 reaches four distinct failures**:
//!
//! - Failure to FILE return
//! - Failure to PAY tax due
//! - Failure to SUPPLY INFORMATION required by IRS
//! - Failure to KEEP RECORDS as required
//!
//! **Penalties — § 7203 itself**:
//! - Imprisonment up to 1 YEAR (misdemeanor cap).
//! - Fine up to $25,000 individual / $100,000 corporation under
//!   § 7203's original text.
//! - 18 U.S.C. § 3571 (Criminal Fines Improvement Act)
//!   supersedes to $100,000 individual / $200,000 corporation.
//! - BOTH imprisonment and fine permitted.
//! - Costs of prosecution.
//!
//! **§ 6050I felony exception**. Where the willful violation
//! involves § 6050I (cash transaction reporting, $10,000
//! threshold), the misdemeanor is ELEVATED to FELONY:
//! - Imprisonment up to 5 YEARS
//! - Same § 3571 fine framework
//!
//! **Spies elevation to § 7201**. A willful failure coupled
//! with AFFIRMATIVE ACTS of concealment (double set of books,
//! false entries, destruction of records, concealment of
//! assets, covering up sources of income, handling affairs to
//! avoid usual records — see `section_7201`) elevates the
//! charge from § 7203 misdemeanor to § 7201 felony. Government
//! routinely charges § 7201 whenever affirmative-act indicia
//! exist.
//!
//! **Cheek defense — Cheek v. United States, 498 U.S. 192
//! (1991)**. A GENUINE GOOD-FAITH BELIEF that one isn't
//! required to file negates the willfulness element, EVEN IF
//! THAT BELIEF IS OBJECTIVELY UNREASONABLE. Subjective belief
//! test. NOT a defense for disagreement with the law
//! (constitutional challenges, tax-protester arguments). Same
//! Cheek defense available as for § 7201 and § 7206.
//!
//! **§ 6531 criminal SOL** — 6 YEARS for § 7203.
//!
//! **Parallel civil penalties**. § 7203 prosecution does NOT
//! bar civil § 6651(a)(1) failure-to-file penalty (5% per
//! month, up to 25%) or § 6651(a)(2) failure-to-pay penalty
//! (0.5% per month). Spies-Daly doctrine permits parallel
//! civil + criminal. § 6501(c)(3) UNLIMITED ASED for no-return-
//! filed cases.
//!
//! **§ 7491 burden shifts do NOT apply to criminal cases** —
//! government bears beyond-reasonable-doubt burden.
//!
//! Citations: IRC § 7203 willful failure to file/pay/supply
//! information/keep records; § 6050I felony elevation for cash
//! reporting violations; § 6531 criminal statute of limitations
//! (6 years for § 7203); 18 U.S.C. § 3571 Criminal Fines
//! Improvement Act; Cheek v. United States, 498 U.S. 192
//! (1991) subjective good-faith defense; Spies v. United
//! States, 317 U.S. 492 (1943) affirmative-act doctrine
//! distinguishing § 7203 from § 7201; § 6651(a)(1) / (a)(2)
//! civil parallel penalties; § 6501(c)(3) unlimited ASED for
//! no-return-filed; § 7491 burden shifts excluded for criminal
//! cases; IRM 9.1.3.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    Individual,
    Corporation,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FailureType {
    FailureToFile,
    FailureToPay,
    FailureToSupplyInformation,
    FailureToKeepRecords,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section7203Input {
    pub entity_type: EntityType,
    pub failure_type: FailureType,
    /// Element 1 — person required by law to file / pay / supply
    /// information / keep records.
    pub element_1_required_by_law: bool,
    /// Element 2 — failure to file / pay / supply information /
    /// keep records at the time required.
    pub element_2_failure_at_time_required: bool,
    /// Element 3 — willfulness (voluntary intentional violation
    /// of known duty).
    pub element_3_willful_voluntary_intentional: bool,
    /// Whether the violation involves § 6050I (cash transaction
    /// reporting, $10,000 threshold) — elevates misdemeanor to
    /// felony.
    pub section_6050i_violation: bool,
    /// Whether affirmative acts of concealment are coupled with
    /// the omission — elevates from § 7203 misdemeanor to § 7201
    /// felony under Spies doctrine.
    pub affirmative_acts_coupled_with_omission: bool,
    /// Whether the Cheek defense (good-faith subjective belief)
    /// is asserted.
    pub cheek_defense_asserted: bool,
    /// Whether the Cheek defense is successful.
    pub cheek_defense_successful: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section7203Result {
    pub prosecution_authorized: bool,
    pub maximum_imprisonment_years: u32,
    pub maximum_fine_cents: i64,
    pub three_elements_satisfied_count: u32,
    /// Whether § 6050I felony exception is engaged (5 years
    /// imprisonment for cash reporting violations).
    pub section_6050i_felony_engaged: bool,
    /// Whether the prosecution should be ELEVATED to § 7201
    /// under Spies doctrine (affirmative acts coupled with
    /// omission).
    pub elevation_to_section_7201_warranted: bool,
    pub cheek_defense_engaged_and_successful: bool,
    pub criminal_sol_years: u32,
    /// Whether parallel § 6651(a)(1) civil failure-to-file
    /// penalty is available.
    pub parallel_civil_6651_a_1_failure_to_file_available: bool,
    /// Whether parallel § 6651(a)(2) civil failure-to-pay
    /// penalty is available.
    pub parallel_civil_6651_a_2_failure_to_pay_available: bool,
    /// Whether § 6501(c)(3) UNLIMITED ASED applies (no-return
    /// filed).
    pub unlimited_ased_no_return_filed: bool,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section7203Input) -> Section7203Result {
    let mut notes: Vec<String> = Vec::new();

    let max_imprisonment_years = if input.section_6050i_violation { 5 } else { 1 };
    let max_fine = match input.entity_type {
        EntityType::Individual => 10_000_000i64,
        EntityType::Corporation => 20_000_000i64,
    };

    notes.push(
        "§ 7203 — criminal MISDEMEANOR (1-year imprisonment cap); distinct from § 7201 5-year felony (requires affirmative acts) and § 7206 3-year felony (fraud and false statements / tax perjury)"
            .to_string(),
    );

    notes.push(
        "18 U.S.C. § 3571 (Criminal Fines Improvement Act) — maximum fines for § 7203 violations: $100,000 individual / $200,000 corporation; supersedes § 7203's original $25,000 / $100,000 caps"
            .to_string(),
    );

    notes.push(
        "§ 6531 criminal statute of limitations — 6 years for § 7203 willful failure to file/pay/supply information"
            .to_string(),
    );

    notes.push(format!(
        "§ 7203 reaches four distinct failures: failure to FILE return + failure to PAY tax + failure to SUPPLY INFORMATION + failure to KEEP RECORDS (this case: {})",
        match input.failure_type {
            FailureType::FailureToFile => "failure to file",
            FailureType::FailureToPay => "failure to pay",
            FailureType::FailureToSupplyInformation => "failure to supply information",
            FailureType::FailureToKeepRecords => "failure to keep records",
        }
    ));

    if input.section_6050i_violation {
        notes.push(
            "§ 6050I felony exception — willful violation of § 6050I cash transaction reporting (over $10,000) ELEVATES misdemeanor to FELONY: 5 YEARS imprisonment + same fine framework"
                .to_string(),
        );
    }

    if input.affirmative_acts_coupled_with_omission {
        notes.push(
            "Spies v. United States, 317 U.S. 492 (1943) — willful omission COUPLED with affirmative acts (concealment, double books, false entries, destruction of records, hiding assets) elevates charge from § 7203 misdemeanor to § 7201 felony; government routinely charges § 7201 when affirmative-act indicia present"
                .to_string(),
        );
    } else {
        notes.push(
            "Spies v. United States, 317 U.S. 492 (1943) — mere willful omission (failure to file/pay/supply) does NOT per se constitute attempt to evade under § 7201; affirmative acts of concealment required to elevate to felony"
                .to_string(),
        );
    }

    let cheek_engaged = input.cheek_defense_asserted && input.cheek_defense_successful;
    if input.cheek_defense_asserted {
        notes.push(
            "Cheek v. United States, 498 U.S. 192 (1991) — genuine good-faith subjective belief that one isn't required to file negates willfulness EVEN IF OBJECTIVELY UNREASONABLE; subjective belief test; NOT a defense for disagreement with the law (constitutional challenges, tax-protester arguments)"
                .to_string(),
        );
        if cheek_engaged {
            notes.push(
                "Cheek defense SUCCESSFUL — subjective good-faith belief established; willfulness element of § 7203 defeated"
                    .to_string(),
            );
        }
    }

    let count = [
        input.element_1_required_by_law,
        input.element_2_failure_at_time_required,
        input.element_3_willful_voluntary_intentional,
    ]
    .iter()
    .filter(|&&b| b)
    .count() as u32;

    let all_three = count == 3;
    let prosecution_authorized = all_three && !cheek_engaged;

    notes.push(format!(
        "§ 7203 three-element test: {}/3 satisfied — (1) required by law to file/pay/supply/keep records + (2) failure at time required + (3) willfulness",
        count
    ));

    if prosecution_authorized {
        notes.push(
            "§ 7203 — all three elements satisfied; misdemeanor prosecution AUTHORIZED"
                .to_string(),
        );
    } else if !all_three {
        notes.push(
            "§ 7203 — government has not established all three elements beyond reasonable doubt; prosecution NOT authorized"
                .to_string(),
        );
    } else if cheek_engaged {
        notes.push(
            "§ 7203 — Cheek defense defeats willfulness element; prosecution NOT authorized despite other elements satisfied"
                .to_string(),
        );
    }

    notes.push(
        "Spies-Daly doctrine — § 7203 prosecution does NOT bar parallel civil penalties; § 6651(a)(1) failure-to-file (5% per month up to 25%) + § 6651(a)(2) failure-to-pay (0.5% per month) routinely apply alongside criminal charge"
            .to_string(),
    );

    let unlimited_ased = matches!(input.failure_type, FailureType::FailureToFile)
        && input.element_2_failure_at_time_required;
    if unlimited_ased {
        notes.push(
            "§ 6501(c)(3) — UNLIMITED ASED for cases where no return is filed; 3-year clock starts only upon actual filing; failure-to-file scenarios remain subject to assessment indefinitely"
                .to_string(),
        );
    }

    notes.push(
        "§ 7491 burden of proof shifts do NOT apply to criminal prosecutions; government bears BEYOND REASONABLE DOUBT burden on each element"
            .to_string(),
    );

    notes.push(
        "IRM 9.1.3 — Criminal Statutory Provisions and Common Law procedural manual governs IRS Criminal Investigation (CI) referral process to DOJ Tax Division"
            .to_string(),
    );

    Section7203Result {
        prosecution_authorized,
        maximum_imprisonment_years: max_imprisonment_years,
        maximum_fine_cents: max_fine,
        three_elements_satisfied_count: count,
        section_6050i_felony_engaged: input.section_6050i_violation,
        elevation_to_section_7201_warranted: input.affirmative_acts_coupled_with_omission,
        cheek_defense_engaged_and_successful: cheek_engaged,
        criminal_sol_years: 6,
        parallel_civil_6651_a_1_failure_to_file_available: matches!(
            input.failure_type,
            FailureType::FailureToFile
        ),
        parallel_civil_6651_a_2_failure_to_pay_available: matches!(
            input.failure_type,
            FailureType::FailureToPay
        ),
        unlimited_ased_no_return_filed: unlimited_ased,
        citation: "IRC §§ 7203, 6050I, 6531, 6651(a)(1), 6651(a)(2), 6501(c)(3), 7201, 7206; 18 U.S.C. § 3571 (Criminal Fines Improvement Act); Spies v. United States, 317 U.S. 492 (1943); Cheek v. United States, 498 U.S. 192 (1991); IRM 9.1.3",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full_failure_to_file() -> Section7203Input {
        Section7203Input {
            entity_type: EntityType::Individual,
            failure_type: FailureType::FailureToFile,
            element_1_required_by_law: true,
            element_2_failure_at_time_required: true,
            element_3_willful_voluntary_intentional: true,
            section_6050i_violation: false,
            affirmative_acts_coupled_with_omission: false,
            cheek_defense_asserted: false,
            cheek_defense_successful: false,
        }
    }

    #[test]
    fn full_three_elements_authorizes_prosecution() {
        let r = check(&full_failure_to_file());
        assert!(r.prosecution_authorized);
        assert_eq!(r.three_elements_satisfied_count, 3);
        assert_eq!(r.maximum_imprisonment_years, 1);
    }

    #[test]
    fn individual_max_fine_100k() {
        let r = check(&full_failure_to_file());
        assert_eq!(r.maximum_fine_cents, 10_000_000);
    }

    #[test]
    fn corporation_max_fine_200k() {
        let mut i = full_failure_to_file();
        i.entity_type = EntityType::Corporation;
        let r = check(&i);
        assert_eq!(r.maximum_fine_cents, 20_000_000);
    }

    #[test]
    fn missing_required_by_law_defeats_prosecution() {
        let mut i = full_failure_to_file();
        i.element_1_required_by_law = false;
        let r = check(&i);
        assert!(!r.prosecution_authorized);
        assert_eq!(r.three_elements_satisfied_count, 2);
    }

    #[test]
    fn missing_failure_at_time_defeats_prosecution() {
        let mut i = full_failure_to_file();
        i.element_2_failure_at_time_required = false;
        let r = check(&i);
        assert!(!r.prosecution_authorized);
    }

    #[test]
    fn missing_willfulness_defeats_prosecution() {
        let mut i = full_failure_to_file();
        i.element_3_willful_voluntary_intentional = false;
        let r = check(&i);
        assert!(!r.prosecution_authorized);
    }

    #[test]
    fn three_element_truth_table() {
        for e1 in [false, true] {
            for e2 in [false, true] {
                for e3 in [false, true] {
                    let mut i = full_failure_to_file();
                    i.element_1_required_by_law = e1;
                    i.element_2_failure_at_time_required = e2;
                    i.element_3_willful_voluntary_intentional = e3;
                    let r = check(&i);
                    let all_three = e1 && e2 && e3;
                    assert_eq!(r.prosecution_authorized, all_three);
                }
            }
        }
    }

    #[test]
    fn cheek_defense_successful_defeats_prosecution() {
        let mut i = full_failure_to_file();
        i.cheek_defense_asserted = true;
        i.cheek_defense_successful = true;
        let r = check(&i);
        assert!(!r.prosecution_authorized);
        assert!(r.cheek_defense_engaged_and_successful);
        assert!(r.notes.iter().any(|n| n.contains("Cheek defense SUCCESSFUL")));
    }

    #[test]
    fn cheek_defense_unsuccessful_does_not_defeat() {
        let mut i = full_failure_to_file();
        i.cheek_defense_asserted = true;
        i.cheek_defense_successful = false;
        let r = check(&i);
        assert!(r.prosecution_authorized);
        assert!(!r.cheek_defense_engaged_and_successful);
    }

    #[test]
    fn cheek_case_objectively_unreasonable_belief_note() {
        let mut i = full_failure_to_file();
        i.cheek_defense_asserted = true;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("Cheek v. United States") && n.contains("OBJECTIVELY UNREASONABLE")));
    }

    #[test]
    fn section_6050i_felony_engagement_5_year_imprisonment() {
        let mut i = full_failure_to_file();
        i.section_6050i_violation = true;
        let r = check(&i);
        assert!(r.section_6050i_felony_engaged);
        assert_eq!(r.maximum_imprisonment_years, 5);
        assert!(r.notes.iter().any(|n| n.contains("§ 6050I felony exception") && n.contains("5 YEARS")));
    }

    #[test]
    fn no_6050i_violation_1_year_imprisonment() {
        let r = check(&full_failure_to_file());
        assert!(!r.section_6050i_felony_engaged);
        assert_eq!(r.maximum_imprisonment_years, 1);
    }

    #[test]
    fn affirmative_acts_elevation_to_7201_warranted() {
        let mut i = full_failure_to_file();
        i.affirmative_acts_coupled_with_omission = true;
        let r = check(&i);
        assert!(r.elevation_to_section_7201_warranted);
        assert!(r.notes.iter().any(|n| n.contains("Spies v. United States") && n.contains("§ 7201 felony")));
    }

    #[test]
    fn no_affirmative_acts_no_elevation() {
        let r = check(&full_failure_to_file());
        assert!(!r.elevation_to_section_7201_warranted);
        assert!(r.notes.iter().any(|n| n.contains("mere willful omission") && n.contains("does NOT")));
    }

    #[test]
    fn criminal_sol_6_years() {
        let r = check(&full_failure_to_file());
        assert_eq!(r.criminal_sol_years, 6);
        assert!(r.notes.iter().any(|n| n.contains("§ 6531") && n.contains("6 years for § 7203")));
    }

    #[test]
    fn failure_to_file_engages_6651_a_1_parallel() {
        let r = check(&full_failure_to_file());
        assert!(r.parallel_civil_6651_a_1_failure_to_file_available);
        assert!(!r.parallel_civil_6651_a_2_failure_to_pay_available);
    }

    #[test]
    fn failure_to_pay_engages_6651_a_2_parallel() {
        let mut i = full_failure_to_file();
        i.failure_type = FailureType::FailureToPay;
        let r = check(&i);
        assert!(!r.parallel_civil_6651_a_1_failure_to_file_available);
        assert!(r.parallel_civil_6651_a_2_failure_to_pay_available);
    }

    #[test]
    fn failure_to_supply_engages_neither_parallel() {
        let mut i = full_failure_to_file();
        i.failure_type = FailureType::FailureToSupplyInformation;
        let r = check(&i);
        assert!(!r.parallel_civil_6651_a_1_failure_to_file_available);
        assert!(!r.parallel_civil_6651_a_2_failure_to_pay_available);
    }

    #[test]
    fn failure_to_keep_records_engages_neither_parallel() {
        let mut i = full_failure_to_file();
        i.failure_type = FailureType::FailureToKeepRecords;
        let r = check(&i);
        assert!(!r.parallel_civil_6651_a_1_failure_to_file_available);
        assert!(!r.parallel_civil_6651_a_2_failure_to_pay_available);
    }

    #[test]
    fn unlimited_ased_engaged_for_failure_to_file_with_no_return() {
        let r = check(&full_failure_to_file());
        assert!(r.unlimited_ased_no_return_filed);
        assert!(r.notes.iter().any(|n| n.contains("§ 6501(c)(3)") && n.contains("UNLIMITED ASED")));
    }

    #[test]
    fn unlimited_ased_not_engaged_for_failure_to_pay() {
        let mut i = full_failure_to_file();
        i.failure_type = FailureType::FailureToPay;
        let r = check(&i);
        assert!(!r.unlimited_ased_no_return_filed);
    }

    #[test]
    fn spies_daly_parallel_civil_note_present() {
        let r = check(&full_failure_to_file());
        assert!(r.notes.iter().any(|n| n.contains("Spies-Daly doctrine") && n.contains("§ 6651(a)(1)") && n.contains("§ 6651(a)(2)")));
    }

    #[test]
    fn section_7491_burden_shifts_excluded_note() {
        let r = check(&full_failure_to_file());
        assert!(r.notes.iter().any(|n| n.contains("§ 7491") && n.contains("BEYOND REASONABLE DOUBT")));
    }

    #[test]
    fn irm_9_1_3_note_present() {
        let r = check(&full_failure_to_file());
        assert!(r.notes.iter().any(|n| n.contains("IRM 9.1.3")));
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&full_failure_to_file());
        assert!(r.citation.contains("§§ 7203, 6050I, 6531"));
        assert!(r.citation.contains("6651(a)(1)"));
        assert!(r.citation.contains("6651(a)(2)"));
        assert!(r.citation.contains("6501(c)(3)"));
        assert!(r.citation.contains("7201"));
        assert!(r.citation.contains("7206"));
        assert!(r.citation.contains("18 U.S.C. § 3571"));
        assert!(r.citation.contains("Spies v. United States"));
        assert!(r.citation.contains("Cheek v. United States"));
        assert!(r.citation.contains("IRM 9.1.3"));
    }

    #[test]
    fn four_failure_types_routed_correctly() {
        for failure_type in [
            FailureType::FailureToFile,
            FailureType::FailureToPay,
            FailureType::FailureToSupplyInformation,
            FailureType::FailureToKeepRecords,
        ] {
            let mut i = full_failure_to_file();
            i.failure_type = failure_type;
            let r = check(&i);
            assert!(r.prosecution_authorized);
            assert!(!r.citation.is_empty());
        }
    }

    #[test]
    fn cheek_only_defeats_when_successful_and_asserted() {
        for asserted in [false, true] {
            for successful in [false, true] {
                let mut i = full_failure_to_file();
                i.cheek_defense_asserted = asserted;
                i.cheek_defense_successful = successful;
                let r = check(&i);
                let cheek_defeats = asserted && successful;
                assert_eq!(r.cheek_defense_engaged_and_successful, cheek_defeats);
                assert_eq!(r.prosecution_authorized, !cheek_defeats);
            }
        }
    }

    #[test]
    fn three_element_count_increments_per_element() {
        let mut i = full_failure_to_file();
        i.element_1_required_by_law = false;
        i.element_2_failure_at_time_required = false;
        i.element_3_willful_voluntary_intentional = false;
        assert_eq!(check(&i).three_elements_satisfied_count, 0);

        i.element_1_required_by_law = true;
        assert_eq!(check(&i).three_elements_satisfied_count, 1);

        i.element_2_failure_at_time_required = true;
        assert_eq!(check(&i).three_elements_satisfied_count, 2);

        i.element_3_willful_voluntary_intentional = true;
        assert_eq!(check(&i).three_elements_satisfied_count, 3);
    }

    #[test]
    fn cfia_18_usc_3571_supersedes_note() {
        let r = check(&full_failure_to_file());
        assert!(r.notes.iter().any(|n| n.contains("18 U.S.C. § 3571") && n.contains("$100,000")));
    }

    #[test]
    fn distinction_from_7201_note_present() {
        let r = check(&full_failure_to_file());
        assert!(r.notes.iter().any(|n| n.contains("§ 7203 — criminal MISDEMEANOR") && n.contains("§ 7201 5-year felony")));
    }

    #[test]
    fn failure_type_routed_in_note() {
        let mut i = full_failure_to_file();
        i.failure_type = FailureType::FailureToSupplyInformation;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("failure to supply information")));
    }

    #[test]
    fn affirmative_acts_truth_table() {
        for acts in [false, true] {
            let mut i = full_failure_to_file();
            i.affirmative_acts_coupled_with_omission = acts;
            let r = check(&i);
            assert_eq!(r.elevation_to_section_7201_warranted, acts);
        }
    }

    #[test]
    fn section_6050i_truth_table() {
        for violation in [false, true] {
            let mut i = full_failure_to_file();
            i.section_6050i_violation = violation;
            let r = check(&i);
            assert_eq!(r.section_6050i_felony_engaged, violation);
            assert_eq!(r.maximum_imprisonment_years, if violation { 5 } else { 1 });
        }
    }
}
