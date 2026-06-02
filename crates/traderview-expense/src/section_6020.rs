//! IRC § 6020 — Returns prepared for or executed by
//! Secretary. The statutory mechanism by which IRS prepares
//! a substitute return when taxpayer fails to file. Trader-
//! procedural-critical because non-filing trader receives a
//! § 6020(b) Substitute For Return (SFR) with worst-case
//! assumptions (no deductions, no cost basis, no § 475(f)
//! M2M election, no § 1091 wash sale adjustments) AND the
//! SFR does NOT qualify as a "valid return" under the
//! Beard test, so § 6501 assessment statute of limitations
//! **NEVER STARTS RUNNING** — IRS can assess at any time.
//! Companion to `section_6201` (assessment authority — § 6020
//! is the procedural mechanism for § 6201(a)(1) assessment
//! of taxes shown on return when no return filed), § 6501
//! (assessment statute), § 6651 (failure-to-file penalty),
//! § 7203 (willful failure to file criminal), § 7202
//! (willful failure to collect tax criminal), § 6212 SNOD
//! (taxpayer deficiency notice after SFR).
//!
//! **§ 6020(a) Preparation of return by Secretary** — if any
//! person shall fail to make a return required by internal
//! revenue laws OR by regulation, but shall **consent to
//! disclose all information necessary for the preparation
//! thereof**, the Secretary may prepare a return which,
//! being **signed by such person**, may be received as the
//! return of such person.
//!
//! **§ 6020(b)(1) Authority of Secretary to execute return**
//! — if any person **fails to make any return required by
//! any internal revenue law or regulation made thereunder
//! at the time prescribed therefor**, or **makes, willfully
//! or otherwise, a false or fraudulent return**, the
//! Secretary shall make such return from his own knowledge
//! and from such information as he can obtain through
//! testimony or otherwise.
//!
//! **§ 6020(b)(2) Status of returns** — any return so made
//! and subscribed by the Secretary shall be **prima facie
//! good and sufficient for all legal purposes**.
//!
//! **§ 6020 vs § 6020(b) distinction**:
//! - **§ 6020(a)** — VOLUNTARY by taxpayer (consents +
//!   discloses + signs); SFR signed by taxpayer counts as a
//!   "return filed by taxpayer" and starts § 6501 ASED.
//! - **§ 6020(b)** — INVOLUNTARY (Secretary's unilateral
//!   action); SFR is **NOT** a "valid return" under the
//!   four-prong Beard test (Beard v. Commissioner, 793 F.2d
//!   139 (6th Cir. 1986)); § 6501 ASED **NEVER STARTS**;
//!   IRS may assess at any time forever.
//!
//! **§ 6020(b) SFR Beard-test failure** — Beard v.
//! Commissioner requires for "valid return":
//! 1. Sufficient data to calculate tax liability;
//! 2. Purports to be a return;
//! 3. Honest and reasonable attempt to satisfy tax laws;
//!    AND
//! 4. **Executed under penalties of perjury** by taxpayer.
//!
//! § 6020(b) SFR fails prong 4 because IRS — not taxpayer —
//! signs; therefore § 6501 ASED never begins. Filing a
//! late-filed but valid return AFTER receiving an SFR
//! starts the § 6501 ASED clock.
//!
//! **§ 6020(b) Regulatory Form 13496 certification** — Form
//! 13496 documents that a document signed by an authorized
//! IRS officer or employee constitutes a § 6020(b) return.
//! Must identify taxpayer by name and ID number, contain
//! sufficient information to compute tax liability, and
//! purport to be a return.
//!
//! **Trader-relevant SFR worst-case computations**:
//! - **No deductions** — standard deduction only; no
//!   Schedule A; no Schedule C trader business expenses; no
//!   § 162 ordinary and necessary; no § 475(f) M2M
//!   election; no § 1091 wash sale loss adjustments.
//! - **No cost basis** on securities — 1099-B gross
//!   proceeds become 100% gain.
//! - **No § 1256 60/40 treatment** — full ordinary income.
//! - **No § 988 currency electious** — full ordinary income.
//! - **No § 199A QBI deduction**.
//! - **No § 1411 NIIT carve-outs**.
//!
//! Citations: 26 USC § 6020(a)-(b); 26 CFR § 301.6020-1
//! (preparation by Secretary); 26 CFR § 301.6020-1T (Form
//! 13496); Rev. Rul. 2005-59; 73 FR 9189 (Substitute for
//! Return regulations, 2008); Beard v. Commissioner, 82
//! T.C. 766 (1984), aff'd 793 F.2d 139 (6th Cir. 1986);
//! IRM 5.18.2 (Business Returns IRC 6020(b) Processing); §
//! 6201; § 6501; § 6651; § 7203; § 7202; § 6212.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReturnPath {
    /// § 6020(a) — voluntary taxpayer consent + disclosure +
    /// signature; counts as filed return.
    Section6020aVoluntary,
    /// § 6020(b) — involuntary IRS-prepared SFR; does NOT
    /// count as filed return under Beard test.
    Section6020bSubstituteForReturn,
    /// No § 6020 action; normal taxpayer-filed return.
    NoSection6020Action,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6020Input {
    pub return_path: ReturnPath,
    /// Whether taxpayer consented to disclose information
    /// (§ 6020(a) precondition).
    pub taxpayer_consented_to_disclose: bool,
    /// Whether taxpayer signed the prepared return
    /// (§ 6020(a) finalization).
    pub taxpayer_signed_prepared_return: bool,
    /// Whether taxpayer made false or fraudulent return
    /// (§ 6020(b) trigger).
    pub false_or_fraudulent_return: bool,
    /// Whether Form 13496 IRS certification was completed
    /// (§ 6020(b) regulatory requirement).
    pub form_13496_certification: bool,
    /// Whether SFR identifies taxpayer by name and
    /// identification number.
    pub taxpayer_identified: bool,
    /// Whether SFR contains sufficient information to
    /// compute tax liability.
    pub sufficient_information_for_tax_liability: bool,
    /// Whether SFR purports to be a return.
    pub purports_to_be_return: bool,
    /// Whether taxpayer subsequently filed a late but valid
    /// return after SFR (starts § 6501 ASED clock).
    pub late_valid_return_filed_after_sfr: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6020Result {
    pub section_6020a_valid: bool,
    pub section_6020b_valid: bool,
    pub sfr_satisfies_beard_test: bool,
    pub section_6501_ased_clock_started: bool,
    pub assessment_authorized: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section6020Input) -> Section6020Result {
    let mut failure_reasons: Vec<String> = Vec::new();

    let s6020a_valid = matches!(input.return_path, ReturnPath::Section6020aVoluntary)
        && input.taxpayer_consented_to_disclose
        && input.taxpayer_signed_prepared_return;

    if matches!(input.return_path, ReturnPath::Section6020aVoluntary)
        && !input.taxpayer_consented_to_disclose
    {
        failure_reasons.push(
            "26 USC § 6020(a) — Secretary preparation of return requires taxpayer CONSENT TO DISCLOSE all information necessary for preparation".to_string(),
        );
    }

    if matches!(input.return_path, ReturnPath::Section6020aVoluntary)
        && !input.taxpayer_signed_prepared_return
    {
        failure_reasons.push(
            "26 USC § 6020(a) — § 6020(a) return becomes valid only upon taxpayer SIGNATURE; unsigned § 6020(a) return is not the taxpayer's return".to_string(),
        );
    }

    let s6020b_valid = matches!(input.return_path, ReturnPath::Section6020bSubstituteForReturn)
        && input.form_13496_certification
        && input.taxpayer_identified
        && input.sufficient_information_for_tax_liability
        && input.purports_to_be_return;

    if matches!(input.return_path, ReturnPath::Section6020bSubstituteForReturn)
        && !input.form_13496_certification
    {
        failure_reasons.push(
            "26 CFR § 301.6020-1 — § 6020(b) return must be certified by Form 13496 signed by authorized IRS officer or employee".to_string(),
        );
    }

    if matches!(input.return_path, ReturnPath::Section6020bSubstituteForReturn)
        && (!input.taxpayer_identified
            || !input.sufficient_information_for_tax_liability
            || !input.purports_to_be_return)
    {
        failure_reasons.push(
            "26 CFR § 301.6020-1 — § 6020(b) return must (1) identify taxpayer by name and ID number, (2) contain sufficient information to compute tax liability, (3) purport to be a return".to_string(),
        );
    }

    let sfr_beard_satisfies = matches!(input.return_path, ReturnPath::Section6020aVoluntary)
        && input.taxpayer_signed_prepared_return;

    let ased_clock_started = match input.return_path {
        ReturnPath::Section6020aVoluntary => input.taxpayer_signed_prepared_return,
        ReturnPath::Section6020bSubstituteForReturn => input.late_valid_return_filed_after_sfr,
        ReturnPath::NoSection6020Action => true,
    };

    let assessment_authorized = match input.return_path {
        ReturnPath::Section6020aVoluntary => s6020a_valid,
        ReturnPath::Section6020bSubstituteForReturn => s6020b_valid,
        ReturnPath::NoSection6020Action => true,
    };

    let notes: Vec<String> = vec![
        "26 USC § 6020(a) — if person fails to make return but CONSENTS TO DISCLOSE all information necessary for preparation, Secretary may prepare a return which, being SIGNED BY SUCH PERSON, may be received as the return of such person".to_string(),
        "26 USC § 6020(b)(1) — if person fails to make any return required by internal revenue law or regulation at time prescribed OR makes false or fraudulent return, Secretary shall make such return from his own knowledge and from such information as he can obtain through testimony or otherwise".to_string(),
        "26 USC § 6020(b)(2) — any § 6020(b) return made and subscribed by Secretary shall be PRIMA FACIE GOOD AND SUFFICIENT for all legal purposes".to_string(),
        "§ 6020(a) vs § 6020(b) distinction — § 6020(a) VOLUNTARY (taxpayer consents + discloses + signs); SFR signed by taxpayer counts as filed return and starts § 6501 ASED; § 6020(b) INVOLUNTARY (Secretary unilateral); SFR is NOT valid return under Beard test; § 6501 ASED NEVER STARTS — IRS may assess at any time forever".to_string(),
        "Beard test (Beard v. Commissioner, 82 T.C. 766 (1984), aff'd 793 F.2d 139 (6th Cir. 1986)) four-prong valid return test: (1) sufficient data to calculate tax liability; (2) purports to be a return; (3) honest and reasonable attempt to satisfy tax laws; (4) EXECUTED UNDER PENALTIES OF PERJURY by taxpayer — § 6020(b) SFR fails prong 4".to_string(),
        "26 CFR § 301.6020-1 + § 301.6020-1T + Form 13496 — § 6020(b) return must (1) be certified by Form 13496 signed by authorized IRS officer/employee, (2) identify taxpayer by name and ID number, (3) contain sufficient information to compute tax liability, (4) purport to be a return".to_string(),
        "Filing a late-filed but valid return AFTER receiving § 6020(b) SFR starts § 6501 ASED clock — taxpayer can cure SFR with subsequent valid return".to_string(),
        "Trader-relevant SFR worst-case computations: NO Schedule C trader business expenses + NO § 162 deductions + NO § 475(f) M2M election + NO § 1091 wash sale adjustments + NO cost basis (1099-B gross proceeds = 100% gain) + NO § 1256 60/40 + NO § 988 elections + NO § 199A QBI + NO § 1411 NIIT carve-outs".to_string(),
        "IRM 5.18.2 (Business Returns IRC 6020(b) Processing) — internal IRS procedural guidance on SFR preparation, certification, and processing".to_string(),
        "Cross-references: § 6020 is procedural mechanism for § 6201(a)(1) assessment of taxes shown on return; pairs with § 6501 (ASED), § 6651 (failure-to-file penalty), § 7203 (willful failure to file criminal), § 7202 (willful failure to collect criminal), § 6212 (SNOD)".to_string(),
    ];

    Section6020Result {
        section_6020a_valid: s6020a_valid,
        section_6020b_valid: s6020b_valid,
        sfr_satisfies_beard_test: sfr_beard_satisfies
            || matches!(input.return_path, ReturnPath::NoSection6020Action),
        section_6501_ased_clock_started: ased_clock_started,
        assessment_authorized,
        failure_reasons,
        citation: "26 USC § 6020(a)-(b); 26 CFR § 301.6020-1 + § 301.6020-1T; Rev. Rul. 2005-59; 73 FR 9189 (2008); Form 13496; Beard v. Commissioner, 82 T.C. 766 (1984), aff'd 793 F.2d 139 (6th Cir. 1986); IRM 5.18.2; § 6201; § 6501; § 6651; § 7203; § 7202; § 6212",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_base() -> Section6020Input {
        Section6020Input {
            return_path: ReturnPath::Section6020aVoluntary,
            taxpayer_consented_to_disclose: true,
            taxpayer_signed_prepared_return: true,
            false_or_fraudulent_return: false,
            form_13496_certification: true,
            taxpayer_identified: true,
            sufficient_information_for_tax_liability: true,
            purports_to_be_return: true,
            late_valid_return_filed_after_sfr: false,
        }
    }

    #[test]
    fn section_6020a_voluntary_consent_signed_valid() {
        let r = check(&valid_base());
        assert!(r.section_6020a_valid);
        assert!(r.sfr_satisfies_beard_test);
        assert!(r.section_6501_ased_clock_started);
    }

    #[test]
    fn section_6020a_without_consent_fails() {
        let mut i = valid_base();
        i.taxpayer_consented_to_disclose = false;
        let r = check(&i);
        assert!(!r.section_6020a_valid);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6020(a)") && f.contains("CONSENT TO DISCLOSE")));
    }

    #[test]
    fn section_6020a_without_signature_fails() {
        let mut i = valid_base();
        i.taxpayer_signed_prepared_return = false;
        let r = check(&i);
        assert!(!r.section_6020a_valid);
        assert!(!r.section_6501_ased_clock_started);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6020(a)") && f.contains("SIGNATURE")));
    }

    #[test]
    fn section_6020b_with_form_13496_and_identification_valid() {
        let mut i = valid_base();
        i.return_path = ReturnPath::Section6020bSubstituteForReturn;
        let r = check(&i);
        assert!(r.section_6020b_valid);
    }

    #[test]
    fn section_6020b_sfr_does_not_satisfy_beard_test() {
        let mut i = valid_base();
        i.return_path = ReturnPath::Section6020bSubstituteForReturn;
        let r = check(&i);
        assert!(!r.sfr_satisfies_beard_test);
    }

    #[test]
    fn section_6020b_ased_never_starts_without_late_valid_return() {
        let mut i = valid_base();
        i.return_path = ReturnPath::Section6020bSubstituteForReturn;
        i.late_valid_return_filed_after_sfr = false;
        let r = check(&i);
        assert!(!r.section_6501_ased_clock_started);
    }

    #[test]
    fn section_6020b_late_valid_return_starts_ased() {
        let mut i = valid_base();
        i.return_path = ReturnPath::Section6020bSubstituteForReturn;
        i.late_valid_return_filed_after_sfr = true;
        let r = check(&i);
        assert!(r.section_6501_ased_clock_started);
    }

    #[test]
    fn section_6020b_without_form_13496_fails() {
        let mut i = valid_base();
        i.return_path = ReturnPath::Section6020bSubstituteForReturn;
        i.form_13496_certification = false;
        let r = check(&i);
        assert!(!r.section_6020b_valid);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 301.6020-1") && f.contains("Form 13496")));
    }

    #[test]
    fn section_6020b_without_taxpayer_identification_fails() {
        let mut i = valid_base();
        i.return_path = ReturnPath::Section6020bSubstituteForReturn;
        i.taxpayer_identified = false;
        let r = check(&i);
        assert!(!r.section_6020b_valid);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("identify taxpayer")));
    }

    #[test]
    fn section_6020b_without_sufficient_info_fails() {
        let mut i = valid_base();
        i.return_path = ReturnPath::Section6020bSubstituteForReturn;
        i.sufficient_information_for_tax_liability = false;
        let r = check(&i);
        assert!(!r.section_6020b_valid);
    }

    #[test]
    fn section_6020b_without_purports_to_be_return_fails() {
        let mut i = valid_base();
        i.return_path = ReturnPath::Section6020bSubstituteForReturn;
        i.purports_to_be_return = false;
        let r = check(&i);
        assert!(!r.section_6020b_valid);
    }

    #[test]
    fn no_section_6020_action_ased_starts_normally() {
        let mut i = valid_base();
        i.return_path = ReturnPath::NoSection6020Action;
        let r = check(&i);
        assert!(r.section_6501_ased_clock_started);
        assert!(r.sfr_satisfies_beard_test);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&valid_base());
        assert!(r.citation.contains("§ 6020(a)-(b)"));
        assert!(r.citation.contains("§ 301.6020-1"));
        assert!(r.citation.contains("§ 301.6020-1T"));
        assert!(r.citation.contains("Rev. Rul. 2005-59"));
        assert!(r.citation.contains("73 FR 9189"));
        assert!(r.citation.contains("Form 13496"));
        assert!(r.citation.contains("Beard v. Commissioner"));
        assert!(r.citation.contains("82 T.C. 766"));
        assert!(r.citation.contains("793 F.2d 139"));
        assert!(r.citation.contains("IRM 5.18.2"));
        assert!(r.citation.contains("§ 6201"));
        assert!(r.citation.contains("§ 6501"));
        assert!(r.citation.contains("§ 6651"));
        assert!(r.citation.contains("§ 7203"));
    }

    #[test]
    fn note_pins_section_a_consent_and_signature_framework() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6020(a)")
            && n.contains("CONSENTS TO DISCLOSE")
            && n.contains("SIGNED BY SUCH PERSON")));
    }

    #[test]
    fn note_pins_section_b1_authority_failure_to_file_or_fraudulent() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6020(b)(1)")
            && n.contains("fails to make any return")
            && n.contains("false or fraudulent return")));
    }

    #[test]
    fn note_pins_section_b2_prima_facie_status() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6020(b)(2)")
            && n.contains("PRIMA FACIE GOOD AND SUFFICIENT")));
    }

    #[test]
    fn note_pins_6020a_vs_6020b_distinction_ased_never_starts() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6020(a) vs § 6020(b)")
            && n.contains("§ 6501 ASED NEVER STARTS")));
    }

    #[test]
    fn note_pins_beard_test_four_prongs() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("Beard test")
            && n.contains("82 T.C. 766")
            && n.contains("793 F.2d 139")
            && n.contains("EXECUTED UNDER PENALTIES OF PERJURY")));
    }

    #[test]
    fn note_pins_form_13496_certification() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("Form 13496")
            && n.contains("§ 301.6020-1")));
    }

    #[test]
    fn note_pins_late_filed_return_cures_sfr() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("late-filed")
            && n.contains("§ 6501 ASED clock")
            && n.contains("cure SFR")));
    }

    #[test]
    fn note_pins_trader_sfr_worst_case_computations() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("Schedule C trader")
            && n.contains("§ 475(f) M2M")
            && n.contains("§ 1091 wash sale")
            && n.contains("NO cost basis")
            && n.contains("§ 199A QBI")));
    }

    #[test]
    fn note_pins_cross_reference_constellation() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6201(a)(1)")
            && n.contains("§ 6501")
            && n.contains("§ 6651")
            && n.contains("§ 7203")
            && n.contains("§ 6212")));
    }

    #[test]
    fn return_path_truth_table_three_cells() {
        for (path, exp_ased_starts) in [
            (ReturnPath::Section6020aVoluntary, true),
            (ReturnPath::Section6020bSubstituteForReturn, false),
            (ReturnPath::NoSection6020Action, true),
        ] {
            let mut i = valid_base();
            i.return_path = path;
            let r = check(&i);
            assert_eq!(
                r.section_6501_ased_clock_started, exp_ased_starts,
                "path={:?} expected ased_starts={}",
                path, exp_ased_starts
            );
        }
    }

    #[test]
    fn section_6020b_ased_only_cures_with_late_valid_return_invariant() {
        let mut i_no_cure = valid_base();
        i_no_cure.return_path = ReturnPath::Section6020bSubstituteForReturn;
        i_no_cure.late_valid_return_filed_after_sfr = false;
        let r_no_cure = check(&i_no_cure);
        assert!(!r_no_cure.section_6501_ased_clock_started);

        let mut i_cured = valid_base();
        i_cured.return_path = ReturnPath::Section6020bSubstituteForReturn;
        i_cured.late_valid_return_filed_after_sfr = true;
        let r_cured = check(&i_cured);
        assert!(r_cured.section_6501_ased_clock_started);
    }

    #[test]
    fn multiple_section_6020b_failures_stack() {
        let mut i = valid_base();
        i.return_path = ReturnPath::Section6020bSubstituteForReturn;
        i.form_13496_certification = false;
        i.taxpayer_identified = false;
        let r = check(&i);
        assert!(!r.section_6020b_valid);
        assert!(r.failure_reasons.len() >= 2);
    }

    #[test]
    fn section_6020a_signature_required_for_ased_invariant() {
        let mut i_signed = valid_base();
        i_signed.taxpayer_signed_prepared_return = true;
        let r_signed = check(&i_signed);
        assert!(r_signed.section_6501_ased_clock_started);

        let mut i_unsigned = valid_base();
        i_unsigned.taxpayer_signed_prepared_return = false;
        let r_unsigned = check(&i_unsigned);
        assert!(!r_unsigned.section_6501_ased_clock_started);
    }
}
