//! IRC § 6531 — Periods of limitation on criminal prosecutions.
//! Cross-cutting reference statute that determines the criminal
//! statute of limitations for ALL criminal tax prosecutions
//! under Title 26. Pairs with `section_7201`, `section_7202`,
//! `section_7203`, `section_7206`, `section_7212`, and other
//! criminal statutes. Critical trader-tax-defense statute —
//! § 6531 SOL determines whether IRS Criminal Investigation
//! referral to DOJ Tax Division can result in prosecution.
//!
//! **§ 6531 general rule** — 3 YEARS from commission of the
//! offense.
//!
//! **§ 6531 exceptions — 6-YEAR SOL** applies to:
//!
//! - § 7201 tax evasion
//! - § 7202 willful failure to collect, account for, or pay over
//!   tax
//! - § 7203 willful failure to FILE return or PAY tax (but NOT
//!   failure to keep records or failure to supply information)
//! - § 7206(1) filing false return / tax perjury
//! - § 7206(2) aid or assist in preparation of false return
//! - § 7207 fraudulent returns, statements, or other documents
//! - § 7212(b) corrupt or forcible interference with seized
//!   property (rescue of property)
//! - § 7214 unlawful acts of revenue officers
//! - Defraud the United States in any manner under 18 U.S.C.
//!   § 371 (Klein conspiracy)
//!
//! **§ 6531 — 3-year SOL** applies to:
//!
//! - § 7203 failure to keep records
//! - § 7203 failure to supply information
//! - § 7205 false withholding exemption certificate
//! - § 7206(3), (4), (5) other subsections
//! - § 7212(a) general obstruction of administration
//! - All other Title 26 offenses not enumerated for 6-year
//!
//! **§ 6531(4) carveout** — the 6-year rule for failure to file
//! a return does NOT apply to returns required to be filed
//! under Part III of Subchapter A of Chapter 61 (partnership
//! returns Form 1065 + exempt organization returns Form 990 +
//! S-corporation returns Form 1120-S). 3-year SOL applies to
//! these returns.
//!
//! **§ 6531 final-paragraph extension** — if the defendant is
//! OUTSIDE the United States or is a fugitive from justice, SOL
//! is TOLLED until 6 months after defendant returns or
//! surrenders.
//!
//! **Commencement of SOL period** — runs from "commission of
//! the offense." For multi-act or continuing offenses (e.g.,
//! tax evasion under § 7201), SOL begins from the LAST
//! affirmative act of evasion (Toussie v. United States, 397
//! U.S. 112 (1970) continuing-offense doctrine narrowed but
//! affirmative-act-doctrine cases survive).
//!
//! **Reset events**:
//! - New affirmative act under § 7201 resets SOL from each act
//! - Concealment of offense resets SOL when concealment ends
//! - Filing of false return resets SOL from date of new return
//!
//! Citations: IRC § 6531 covers periods of limitation on
//! criminal prosecutions. § 6531(1) sets the general 3-year
//! rule. § 6531(2) provides the 6-year exception covering
//! § 7201 and § 7203 and § 7206(1) and (2) and § 7207 and
//! § 7212(b). § 6531(4) carves out the Part III Subchapter A
//! Chapter 61 partnership / exempt org / S-corp return cases.
//! The § 6531 final paragraph governs outside-US and fugitive
//! tolling. Toussie v. United States, 397 U.S. 112 (1970)
//! addresses continuing-offense doctrine. DOJ Criminal Tax
//! Manual § 7.00 and IRM 25.6.2.1 govern Statute of Limitations
//! procedural framework.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CriminalTaxOffense {
    Section7201TaxEvasion,
    Section7202TrustFundFailure,
    Section7203FailureToFileOrPay,
    Section7203FailureToKeepRecords,
    Section7203FailureToSupplyInformation,
    Section7205FalseWithholdingExemption,
    Section7206_1FilingFalseReturn,
    Section7206_2AidAssistFalseReturn,
    Section7206_3FraudulentBondsPermitsEntries,
    Section7206_4ConcealmentRemoval,
    Section7206_5CompromisesClosingAgreements,
    Section7207FraudulentReturnsStatements,
    Section7212aObstructionGeneral,
    Section7212bRescueOfSeizedProperty,
    Section7214UnlawfulActsRevenueOfficers,
    KleinConspiracy18Usc371,
    OtherTitle26Offense,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6531Input {
    pub offense: CriminalTaxOffense,
    /// Years since commission of the offense (from last
    /// affirmative act for continuing offenses).
    pub years_since_offense_commission: u32,
    /// Whether the offense involves a partnership return / exempt
    /// organization return / S-corp return — triggers § 6531(4)
    /// carveout from 6-year to 3-year for failure to file.
    pub partnership_exempt_org_or_s_corp_return: bool,
    /// Whether the defendant has been OUTSIDE the United States
    /// or is a fugitive — triggers SOL tolling.
    pub defendant_outside_us_or_fugitive: bool,
    /// Whether the defendant has returned or surrendered (6-month
    /// post-return window begins).
    pub defendant_has_returned_or_surrendered: bool,
    /// Months since defendant returned or surrendered.
    pub months_since_return_or_surrender: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6531Result {
    pub applicable_sol_years: u32,
    pub six_year_exception_engaged: bool,
    pub section_6531_4_carveout_engaged: bool,
    pub sol_tolling_for_absence_engaged: bool,
    pub sol_satisfied: bool,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section6531Input) -> Section6531Result {
    let mut notes: Vec<String> = Vec::new();

    notes.push(
        "§ 6531 general rule — 3-year SOL for all criminal prosecutions under Title 26".to_string(),
    );

    let base_six_year = matches!(
        input.offense,
        CriminalTaxOffense::Section7201TaxEvasion
            | CriminalTaxOffense::Section7202TrustFundFailure
            | CriminalTaxOffense::Section7203FailureToFileOrPay
            | CriminalTaxOffense::Section7206_1FilingFalseReturn
            | CriminalTaxOffense::Section7206_2AidAssistFalseReturn
            | CriminalTaxOffense::Section7207FraudulentReturnsStatements
            | CriminalTaxOffense::Section7212bRescueOfSeizedProperty
            | CriminalTaxOffense::Section7214UnlawfulActsRevenueOfficers
            | CriminalTaxOffense::KleinConspiracy18Usc371
    );

    let carveout_6531_4 = base_six_year
        && matches!(
            input.offense,
            CriminalTaxOffense::Section7203FailureToFileOrPay
        )
        && input.partnership_exempt_org_or_s_corp_return;

    let six_year_engaged = base_six_year && !carveout_6531_4;

    let applicable_sol = if six_year_engaged { 6 } else { 3 };

    notes.push(format!(
        "§ 6531 — offense {} applicable SOL: {} years",
        match input.offense {
            CriminalTaxOffense::Section7201TaxEvasion => "§ 7201 tax evasion",
            CriminalTaxOffense::Section7202TrustFundFailure => "§ 7202 trust fund failure",
            CriminalTaxOffense::Section7203FailureToFileOrPay => "§ 7203 failure to file or pay",
            CriminalTaxOffense::Section7203FailureToKeepRecords => "§ 7203 failure to keep records",
            CriminalTaxOffense::Section7203FailureToSupplyInformation =>
                "§ 7203 failure to supply information",
            CriminalTaxOffense::Section7205FalseWithholdingExemption =>
                "§ 7205 false withholding exemption",
            CriminalTaxOffense::Section7206_1FilingFalseReturn => "§ 7206(1) filing false return",
            CriminalTaxOffense::Section7206_2AidAssistFalseReturn =>
                "§ 7206(2) aiding false return",
            CriminalTaxOffense::Section7206_3FraudulentBondsPermitsEntries =>
                "§ 7206(3) fraudulent bonds",
            CriminalTaxOffense::Section7206_4ConcealmentRemoval => "§ 7206(4) concealment/removal",
            CriminalTaxOffense::Section7206_5CompromisesClosingAgreements =>
                "§ 7206(5) compromises",
            CriminalTaxOffense::Section7207FraudulentReturnsStatements =>
                "§ 7207 fraudulent returns/statements",
            CriminalTaxOffense::Section7212aObstructionGeneral => "§ 7212(a) obstruction (general)",
            CriminalTaxOffense::Section7212bRescueOfSeizedProperty =>
                "§ 7212(b) rescue of seized property",
            CriminalTaxOffense::Section7214UnlawfulActsRevenueOfficers =>
                "§ 7214 unlawful acts of revenue officers",
            CriminalTaxOffense::KleinConspiracy18Usc371 => "18 U.S.C. § 371 Klein conspiracy",
            CriminalTaxOffense::OtherTitle26Offense => "other Title 26 offense",
        },
        applicable_sol
    ));

    if carveout_6531_4 {
        notes.push(
            "§ 6531(4) carveout engaged — the 6-year rule for failure to file return does NOT apply to returns required to be filed under Part III of Subchapter A of Chapter 61 (partnership Form 1065 + exempt org Form 990 + S-corp Form 1120-S); 3-year SOL applies"
                .to_string(),
        );
    }

    if six_year_engaged {
        notes.push(
            "§ 6531(2) — 6-year exception engaged; § 7201 + § 7202 + § 7203 failure-to-file/pay + § 7206(1) + § 7206(2) + § 7207 + § 7212(b) + § 7214 + Klein conspiracy all carry 6-year SOL"
                .to_string(),
        );
    }

    let absence_tolling = input.defendant_outside_us_or_fugitive
        && !(input.defendant_has_returned_or_surrendered
            && input.months_since_return_or_surrender > 6);

    if input.defendant_outside_us_or_fugitive {
        notes.push(
            "§ 6531 final paragraph — defendant outside the United States or fugitive from justice TOLLS the SOL until 6 months after defendant returns or surrenders"
                .to_string(),
        );

        if absence_tolling {
            notes.push(
                "SOL TOLLING engaged — period of absence + 6-month post-return window has NOT yet expired; prosecution not time-barred"
                    .to_string(),
            );
        } else {
            notes.push(format!(
                "SOL tolling expired — {} months since defendant returned or surrendered (exceeds 6-month window); SOL clock has resumed",
                input.months_since_return_or_surrender
            ));
        }
    }

    let sol_satisfied = absence_tolling || input.years_since_offense_commission < applicable_sol;

    notes.push(format!(
        "{} years since offense commission; {} SOL applicable; prosecution {}",
        input.years_since_offense_commission,
        applicable_sol,
        if sol_satisfied {
            "TIMELY (within SOL or tolled)"
        } else {
            "TIME-BARRED (SOL expired)"
        }
    ));

    notes.push(
        "Toussie v. United States, 397 U.S. 112 (1970) — continuing-offense doctrine narrowed but affirmative-act-doctrine cases survive; SOL runs from LAST affirmative act for multi-act offenses like § 7201 tax evasion"
            .to_string(),
    );

    notes.push(
        "DOJ Criminal Tax Manual § 7.00 + IRM 25.6.2.1 — IRS Criminal Investigation must refer to DOJ Tax Division before SOL expiration; § 6531 SOL is JURISDICTIONAL"
            .to_string(),
    );

    Section6531Result {
        applicable_sol_years: applicable_sol,
        six_year_exception_engaged: six_year_engaged,
        section_6531_4_carveout_engaged: carveout_6531_4,
        sol_tolling_for_absence_engaged: absence_tolling,
        sol_satisfied,
        citation: "IRC § 6531 (general 3-year rule + 6-year exceptions for enumerated offenses + § 6531(4) Part III Subchapter A Chapter 61 carveout + final-paragraph outside-US/fugitive tolling); Toussie v. United States, 397 U.S. 112 (1970); DOJ Criminal Tax Manual § 7.00; IRM 25.6.2.1",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(offense: CriminalTaxOffense, years: u32) -> Section6531Input {
        Section6531Input {
            offense,
            years_since_offense_commission: years,
            partnership_exempt_org_or_s_corp_return: false,
            defendant_outside_us_or_fugitive: false,
            defendant_has_returned_or_surrendered: false,
            months_since_return_or_surrender: 0,
        }
    }

    #[test]
    fn section_7201_6_year_sol() {
        let r = check(&base(CriminalTaxOffense::Section7201TaxEvasion, 5));
        assert_eq!(r.applicable_sol_years, 6);
        assert!(r.six_year_exception_engaged);
        assert!(r.sol_satisfied);
    }

    #[test]
    fn section_7202_6_year_sol() {
        let r = check(&base(CriminalTaxOffense::Section7202TrustFundFailure, 5));
        assert_eq!(r.applicable_sol_years, 6);
    }

    #[test]
    fn section_7203_failure_to_file_or_pay_6_year_sol() {
        let r = check(&base(CriminalTaxOffense::Section7203FailureToFileOrPay, 5));
        assert_eq!(r.applicable_sol_years, 6);
    }

    #[test]
    fn section_7203_failure_to_keep_records_3_year_sol() {
        let r = check(&base(
            CriminalTaxOffense::Section7203FailureToKeepRecords,
            2,
        ));
        assert_eq!(r.applicable_sol_years, 3);
        assert!(!r.six_year_exception_engaged);
    }

    #[test]
    fn section_7203_failure_to_supply_information_3_year_sol() {
        let r = check(&base(
            CriminalTaxOffense::Section7203FailureToSupplyInformation,
            2,
        ));
        assert_eq!(r.applicable_sol_years, 3);
    }

    #[test]
    fn section_7205_3_year_sol() {
        let r = check(&base(
            CriminalTaxOffense::Section7205FalseWithholdingExemption,
            2,
        ));
        assert_eq!(r.applicable_sol_years, 3);
    }

    #[test]
    fn section_7206_1_filing_false_return_6_year_sol() {
        let r = check(&base(CriminalTaxOffense::Section7206_1FilingFalseReturn, 5));
        assert_eq!(r.applicable_sol_years, 6);
    }

    #[test]
    fn section_7206_2_aiding_6_year_sol() {
        let r = check(&base(
            CriminalTaxOffense::Section7206_2AidAssistFalseReturn,
            5,
        ));
        assert_eq!(r.applicable_sol_years, 6);
    }

    #[test]
    fn section_7206_3_4_5_3_year_sol() {
        for offense in [
            CriminalTaxOffense::Section7206_3FraudulentBondsPermitsEntries,
            CriminalTaxOffense::Section7206_4ConcealmentRemoval,
            CriminalTaxOffense::Section7206_5CompromisesClosingAgreements,
        ] {
            let r = check(&base(offense, 2));
            assert_eq!(r.applicable_sol_years, 3);
        }
    }

    #[test]
    fn section_7207_6_year_sol() {
        let r = check(&base(
            CriminalTaxOffense::Section7207FraudulentReturnsStatements,
            5,
        ));
        assert_eq!(r.applicable_sol_years, 6);
    }

    #[test]
    fn section_7212a_general_obstruction_3_year_sol() {
        let r = check(&base(CriminalTaxOffense::Section7212aObstructionGeneral, 2));
        assert_eq!(r.applicable_sol_years, 3);
    }

    #[test]
    fn section_7212b_rescue_property_6_year_sol() {
        let r = check(&base(
            CriminalTaxOffense::Section7212bRescueOfSeizedProperty,
            5,
        ));
        assert_eq!(r.applicable_sol_years, 6);
    }

    #[test]
    fn klein_conspiracy_6_year_sol() {
        let r = check(&base(CriminalTaxOffense::KleinConspiracy18Usc371, 5));
        assert_eq!(r.applicable_sol_years, 6);
    }

    #[test]
    fn other_offense_3_year_sol() {
        let r = check(&base(CriminalTaxOffense::OtherTitle26Offense, 2));
        assert_eq!(r.applicable_sol_years, 3);
    }

    #[test]
    fn section_6531_4_carveout_partnership_return() {
        let mut i = base(CriminalTaxOffense::Section7203FailureToFileOrPay, 4);
        i.partnership_exempt_org_or_s_corp_return = true;
        let r = check(&i);
        assert_eq!(r.applicable_sol_years, 3);
        assert!(r.section_6531_4_carveout_engaged);
        assert!(r.notes.iter().any(|n| n.contains("§ 6531(4) carveout")
            && n.contains("Form 1065")
            && n.contains("Form 990")
            && n.contains("Form 1120-S")));
    }

    #[test]
    fn section_6531_4_carveout_only_applies_to_7203() {
        let mut i = base(CriminalTaxOffense::Section7201TaxEvasion, 4);
        i.partnership_exempt_org_or_s_corp_return = true;
        let r = check(&i);
        assert_eq!(r.applicable_sol_years, 6);
        assert!(!r.section_6531_4_carveout_engaged);
    }

    #[test]
    fn section_6531_4_carveout_truth_table() {
        for partnership in [false, true] {
            let mut i = base(CriminalTaxOffense::Section7203FailureToFileOrPay, 4);
            i.partnership_exempt_org_or_s_corp_return = partnership;
            let r = check(&i);
            assert_eq!(r.applicable_sol_years, if partnership { 3 } else { 6 });
        }
    }

    #[test]
    fn sol_satisfied_within_3_years_for_3_year_offense() {
        let r = check(&base(
            CriminalTaxOffense::Section7205FalseWithholdingExemption,
            2,
        ));
        assert!(r.sol_satisfied);
    }

    #[test]
    fn sol_expired_after_3_years_for_3_year_offense() {
        let r = check(&base(
            CriminalTaxOffense::Section7205FalseWithholdingExemption,
            3,
        ));
        assert!(!r.sol_satisfied);
    }

    #[test]
    fn sol_satisfied_within_6_years_for_6_year_offense() {
        let r = check(&base(CriminalTaxOffense::Section7201TaxEvasion, 5));
        assert!(r.sol_satisfied);
    }

    #[test]
    fn sol_expired_after_6_years_for_6_year_offense() {
        let r = check(&base(CriminalTaxOffense::Section7201TaxEvasion, 6));
        assert!(!r.sol_satisfied);
    }

    #[test]
    fn outside_us_tolling_engages() {
        let mut i = base(CriminalTaxOffense::Section7201TaxEvasion, 10);
        i.defendant_outside_us_or_fugitive = true;
        let r = check(&i);
        assert!(r.sol_tolling_for_absence_engaged);
        assert!(r.sol_satisfied);
    }

    #[test]
    fn returned_within_6_months_still_tolled() {
        let mut i = base(CriminalTaxOffense::Section7201TaxEvasion, 10);
        i.defendant_outside_us_or_fugitive = true;
        i.defendant_has_returned_or_surrendered = true;
        i.months_since_return_or_surrender = 3;
        let r = check(&i);
        assert!(r.sol_tolling_for_absence_engaged);
    }

    #[test]
    fn returned_past_6_months_tolling_ends() {
        let mut i = base(CriminalTaxOffense::Section7201TaxEvasion, 10);
        i.defendant_outside_us_or_fugitive = true;
        i.defendant_has_returned_or_surrendered = true;
        i.months_since_return_or_surrender = 7;
        let r = check(&i);
        assert!(!r.sol_tolling_for_absence_engaged);
    }

    #[test]
    fn returned_at_6_month_boundary_still_tolled() {
        let mut i = base(CriminalTaxOffense::Section7201TaxEvasion, 10);
        i.defendant_outside_us_or_fugitive = true;
        i.defendant_has_returned_or_surrendered = true;
        i.months_since_return_or_surrender = 6;
        let r = check(&i);
        assert!(r.sol_tolling_for_absence_engaged);
    }

    #[test]
    fn general_3_year_rule_note_present() {
        let r = check(&base(CriminalTaxOffense::OtherTitle26Offense, 2));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6531 general rule") && n.contains("3-year SOL")));
    }

    #[test]
    fn six_year_exception_note_lists_enumerated_offenses() {
        let r = check(&base(CriminalTaxOffense::Section7201TaxEvasion, 5));
        assert!(r.notes.iter().any(|n| n.contains("§ 6531(2)")
            && n.contains("§ 7201")
            && n.contains("§ 7203 failure-to-file/pay")));
    }

    #[test]
    fn toussie_continuing_offense_note_present() {
        let r = check(&base(CriminalTaxOffense::Section7201TaxEvasion, 5));
        assert!(r.notes.iter().any(|n| n
            .contains("Toussie v. United States, 397 U.S. 112 (1970)")
            && n.contains("LAST affirmative act")));
    }

    #[test]
    fn doj_manual_irm_note_present() {
        let r = check(&base(CriminalTaxOffense::Section7201TaxEvasion, 5));
        assert!(r.notes.iter().any(|n| n.contains("DOJ Criminal Tax Manual")
            && n.contains("IRM 25.6.2.1")
            && n.contains("JURISDICTIONAL")));
    }

    #[test]
    fn citation_pins_authorities() {
        let r = check(&base(CriminalTaxOffense::Section7201TaxEvasion, 5));
        assert!(r.citation.contains("§ 6531"));
        assert!(r.citation.contains("§ 6531(4)"));
        assert!(r.citation.contains("Toussie v. United States"));
        assert!(r.citation.contains("DOJ Criminal Tax Manual"));
        assert!(r.citation.contains("IRM 25.6.2.1"));
    }

    #[test]
    fn nine_six_year_offenses_truth_table() {
        let six_year_offenses = [
            CriminalTaxOffense::Section7201TaxEvasion,
            CriminalTaxOffense::Section7202TrustFundFailure,
            CriminalTaxOffense::Section7203FailureToFileOrPay,
            CriminalTaxOffense::Section7206_1FilingFalseReturn,
            CriminalTaxOffense::Section7206_2AidAssistFalseReturn,
            CriminalTaxOffense::Section7207FraudulentReturnsStatements,
            CriminalTaxOffense::Section7212bRescueOfSeizedProperty,
            CriminalTaxOffense::Section7214UnlawfulActsRevenueOfficers,
            CriminalTaxOffense::KleinConspiracy18Usc371,
        ];
        for offense in six_year_offenses {
            let r = check(&base(offense, 5));
            assert_eq!(r.applicable_sol_years, 6);
            assert!(r.six_year_exception_engaged);
        }
    }

    #[test]
    fn three_year_offenses_truth_table() {
        let three_year_offenses = [
            CriminalTaxOffense::Section7203FailureToKeepRecords,
            CriminalTaxOffense::Section7203FailureToSupplyInformation,
            CriminalTaxOffense::Section7205FalseWithholdingExemption,
            CriminalTaxOffense::Section7206_3FraudulentBondsPermitsEntries,
            CriminalTaxOffense::Section7206_4ConcealmentRemoval,
            CriminalTaxOffense::Section7206_5CompromisesClosingAgreements,
            CriminalTaxOffense::Section7212aObstructionGeneral,
            CriminalTaxOffense::OtherTitle26Offense,
        ];
        for offense in three_year_offenses {
            let r = check(&base(offense, 2));
            assert_eq!(r.applicable_sol_years, 3);
            assert!(!r.six_year_exception_engaged);
        }
    }

    #[test]
    fn sol_boundary_at_exactly_3_years_expired() {
        let r = check(&base(
            CriminalTaxOffense::Section7205FalseWithholdingExemption,
            3,
        ));
        assert!(!r.sol_satisfied);
    }

    #[test]
    fn sol_boundary_at_exactly_6_years_expired() {
        let r = check(&base(CriminalTaxOffense::Section7201TaxEvasion, 6));
        assert!(!r.sol_satisfied);
    }

    #[test]
    fn final_paragraph_tolling_note_present_when_absent() {
        let mut i = base(CriminalTaxOffense::Section7201TaxEvasion, 5);
        i.defendant_outside_us_or_fugitive = true;
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6531 final paragraph") && n.contains("TOLLS")));
    }

    #[test]
    fn tolling_disengages_when_no_absence() {
        let r = check(&base(CriminalTaxOffense::Section7201TaxEvasion, 5));
        assert!(!r.sol_tolling_for_absence_engaged);
    }
}
