//! IRC § 6672 — Trust Fund Recovery Penalty (TFRP). One of the
//! most severe penalties in the Code: 100% personal liability on
//! responsible persons for unpaid trust fund portion of employment
//! taxes (employee income tax withholding + employee FICA share).
//! The employer FICA match is NOT trust fund. Critical
//! trader-business operational risk for any trader operating
//! through an entity with W-2 employees (LLC with employees,
//! S-corp with shareholder-employees, C-corp).
//!
//! **Two-prong test for TFRP liability:**
//!
//! 1. **Responsible person** — individual with significant (not
//!    necessarily exclusive) control over the company's finances.
//!    Three factors: STATUS (officer, director, member),
//!    DUTY (designated to perform payroll/tax functions),
//!    AUTHORITY (check-signing, vendor payment authority).
//!
//! 2. **Willfulness** — responsible person KNEW trust fund taxes
//!    were due AND either chose not to pay them OR recklessly
//!    disregarded an obvious risk they would not be paid. NO evil
//!    intent or fraudulent purpose required. Using available
//!    funds to pay OTHER creditors when employment taxes
//!    unpaid is a classic indicator of willfulness.
//!
//! **§ 6672(b)(1) — Preliminary notice + 60-day waiting period**.
//! Before assessment, IRS MUST send preliminary written notice
//! (Letter 1153 + Form 2751) at least 60 days before assessment
//! to allow protest / Appeals opportunity.
//!
//! **§ 6672(d) — Right of contribution among multiple
//! responsible persons**. Each responsible person is JOINTLY AND
//! SEVERALLY liable for full TFRP; payor may seek contribution
//! from co-responsible persons (state-law contribution claim).
//!
//! **Bankruptcy implications**: TFRP is **nondischargeable**
//! under 11 U.S.C. § 523(a)(7); priority claim under §
//! 507(a)(8)(C). Personal bankruptcy does NOT eliminate TFRP
//! liability.
//!
//! **Trust fund taxes** (subject to TFRP):
//! - Federal income tax withheld from employee wages (§ 3402)
//! - Employee share of FICA (§ 3101)
//!
//! **NOT trust fund taxes** (not subject to TFRP):
//! - Employer share of FICA (§ 3111)
//! - FUTA (§ 3301)
//! - Sales and excise taxes (separately collected but not
//!   classified as trust fund for § 6672 purposes)
//!
//! Citations: IRC § 6672(a) covers the 100% penalty plus the
//! responsible-person and willfulness prongs. § 6672(b)(1) covers
//! the preliminary notice and 60-day waiting period. § 6672(d)
//! governs the right of contribution. Supporting sections: § 3402
//! income tax withholding, § 3101 employee FICA, and § 3111
//! employer FICA which is NOT trust fund. IRM 8.25.1 is the TFRP
//! procedural manual. 11 U.S.C. § 523(a)(7) makes TFRP
//! nondischargeable in bankruptcy and 11 U.S.C. § 507(a)(8)(C)
//! gives it priority-claim status.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Section6672Input {
    /// Whether the individual has significant (not necessarily
    /// exclusive) control over the company's finances —
    /// satisfies the "responsible person" prong.
    pub significant_control_over_company_finances: bool,
    /// Whether the individual is an officer, director, member,
    /// or designated payroll/tax-function person (status prong).
    pub has_officer_director_or_designated_status: bool,
    /// Whether the individual has check-signing or vendor-payment
    /// authority (authority prong).
    pub has_check_signing_or_payment_authority: bool,
    /// Whether the individual KNEW trust fund taxes were due
    /// (willfulness prong — knowledge).
    pub knew_trust_fund_taxes_were_due: bool,
    /// Whether the individual recklessly disregarded an obvious
    /// risk that trust fund taxes would not be paid (willfulness
    /// prong — reckless disregard alternative).
    pub recklessly_disregarded_obvious_risk: bool,
    /// Whether the individual used available company funds to
    /// pay OTHER creditors when employment taxes were unpaid
    /// (classic willfulness indicator).
    pub used_funds_to_pay_other_creditors: bool,
    /// Whether the IRS sent the § 6672(b)(1) preliminary notice
    /// (Letter 1153 + Form 2751) at least 60 days before
    /// assessment.
    pub preliminary_notice_60_day_window_provided: bool,
    /// Whether the responsible person is one of multiple
    /// responsible persons (triggers § 6672(d) contribution
    /// rights).
    pub multiple_responsible_persons_involved: bool,
    /// Whether the responsible person is attempting to discharge
    /// the TFRP in personal bankruptcy.
    pub bankruptcy_discharge_attempted: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6672Result {
    /// Whether the individual is a "responsible person" under
    /// § 6672 — satisfied by any combination of significant
    /// control, status, or authority.
    pub responsible_person_status: bool,
    /// Whether the willfulness prong is engaged.
    pub willfulness_engaged: bool,
    /// Whether the TFRP is imposable (both prongs satisfied).
    pub tfrp_imposable: bool,
    /// Whether the § 6672(b)(1) preliminary-notice procedural
    /// requirement is satisfied.
    pub preliminary_notice_compliance: bool,
    /// Whether the § 6672(b)(1) violation makes the assessment
    /// procedurally defective.
    pub preliminary_notice_violation: bool,
    /// Whether the right of contribution under § 6672(d) is
    /// engaged among multiple responsible persons.
    pub contribution_right_engaged: bool,
    /// Whether the TFRP is nondischargeable in bankruptcy
    /// (always YES per 11 U.S.C. § 523(a)(7)).
    pub nondischargeable_in_bankruptcy: bool,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section6672Input) -> Section6672Result {
    let mut notes: Vec<String> = Vec::new();

    let responsible_person = input.significant_control_over_company_finances
        || input.has_officer_director_or_designated_status
        || input.has_check_signing_or_payment_authority;

    let willfulness = input.knew_trust_fund_taxes_were_due
        || input.recklessly_disregarded_obvious_risk
        || input.used_funds_to_pay_other_creditors;

    let tfrp_imposable = responsible_person && willfulness;

    let preliminary_notice_compliance = input.preliminary_notice_60_day_window_provided;
    let preliminary_notice_violation = tfrp_imposable && !preliminary_notice_compliance;

    let contribution_right_engaged = tfrp_imposable && input.multiple_responsible_persons_involved;

    notes.push(
        "§ 6672(a) — 100% PERSONAL liability on responsible persons for unpaid trust fund portion of employment taxes; one of the most severe penalties in the Code"
            .to_string(),
    );

    notes.push(
        "trust fund taxes (subject to TFRP): § 3402 income tax withholding + § 3101 employee FICA share; NOT trust fund: § 3111 employer FICA match + § 3301 FUTA"
            .to_string(),
    );

    if responsible_person {
        notes.push(
            "§ 6672 responsible person prong satisfied — significant (not exclusive) control over finances OR officer/director/designated status OR check-signing/payment authority"
                .to_string(),
        );
    } else {
        notes.push(
            "§ 6672 responsible person prong NOT satisfied — individual lacks significant control, status, or authority"
                .to_string(),
        );
    }

    if willfulness {
        notes.push(
            "§ 6672 willfulness prong satisfied — no evil intent required; knowledge OR reckless disregard suffices; using available funds to pay OTHER creditors when employment taxes unpaid is classic willfulness indicator"
                .to_string(),
        );
    } else {
        notes.push(
            "§ 6672 willfulness prong NOT satisfied — neither knowledge of unpaid taxes nor reckless disregard nor preferential creditor payment established"
                .to_string(),
        );
    }

    if tfrp_imposable {
        notes.push(
            "§ 6672 TFRP IMPOSABLE — both responsible person and willfulness prongs satisfied; 100% of trust fund portion of unpaid employment taxes attaches PERSONALLY"
                .to_string(),
        );
    }

    if preliminary_notice_violation {
        notes.push(
            "§ 6672(b)(1) — IRS MUST send preliminary written notice (Letter 1153 + Form 2751) at least 60 days before assessment; failure to comply renders assessment procedurally defective"
                .to_string(),
        );
    } else if preliminary_notice_compliance && tfrp_imposable {
        notes.push(
            "§ 6672(b)(1) — preliminary notice + 60-day waiting period satisfied; taxpayer had opportunity to protest / Appeals review"
                .to_string(),
        );
    }

    if contribution_right_engaged {
        notes.push(
            "§ 6672(d) — each responsible person is JOINTLY AND SEVERALLY liable for full TFRP; paying responsible person may seek contribution from co-responsible persons via state-law contribution claim"
                .to_string(),
        );
    }

    if input.bankruptcy_discharge_attempted {
        notes.push(
            "11 U.S.C. § 523(a)(7) — TFRP is NONDISCHARGEABLE in personal bankruptcy; 11 U.S.C. § 507(a)(8)(C) — priority claim; personal bankruptcy does NOT eliminate TFRP liability"
                .to_string(),
        );
    }

    notes.push(
        "IRM 8.25.1 — TFRP procedural manual governs IRS Appeals procedure; assessment must follow preliminary-notice protest opportunity"
            .to_string(),
    );

    Section6672Result {
        responsible_person_status: responsible_person,
        willfulness_engaged: willfulness,
        tfrp_imposable,
        preliminary_notice_compliance,
        preliminary_notice_violation,
        contribution_right_engaged,
        nondischargeable_in_bankruptcy: true,
        citation: "IRC §§ 6672(a), 6672(b)(1), 6672(d), 3402, 3101, 3111, 3301; IRM 8.25.1; 11 U.S.C. §§ 523(a)(7), 507(a)(8)(C)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Section6672Input {
        Section6672Input {
            significant_control_over_company_finances: false,
            has_officer_director_or_designated_status: false,
            has_check_signing_or_payment_authority: false,
            knew_trust_fund_taxes_were_due: false,
            recklessly_disregarded_obvious_risk: false,
            used_funds_to_pay_other_creditors: false,
            preliminary_notice_60_day_window_provided: false,
            multiple_responsible_persons_involved: false,
            bankruptcy_discharge_attempted: false,
        }
    }

    fn full_tfrp() -> Section6672Input {
        Section6672Input {
            significant_control_over_company_finances: true,
            has_officer_director_or_designated_status: true,
            has_check_signing_or_payment_authority: true,
            knew_trust_fund_taxes_were_due: true,
            recklessly_disregarded_obvious_risk: false,
            used_funds_to_pay_other_creditors: true,
            preliminary_notice_60_day_window_provided: true,
            multiple_responsible_persons_involved: false,
            bankruptcy_discharge_attempted: false,
        }
    }

    #[test]
    fn no_responsible_person_no_willfulness_no_tfrp() {
        let r = check(&base());
        assert!(!r.responsible_person_status);
        assert!(!r.willfulness_engaged);
        assert!(!r.tfrp_imposable);
    }

    #[test]
    fn full_tfrp_both_prongs_imposable() {
        let r = check(&full_tfrp());
        assert!(r.responsible_person_status);
        assert!(r.willfulness_engaged);
        assert!(r.tfrp_imposable);
    }

    #[test]
    fn significant_control_alone_satisfies_responsible_person() {
        let mut i = base();
        i.significant_control_over_company_finances = true;
        let r = check(&i);
        assert!(r.responsible_person_status);
    }

    #[test]
    fn officer_status_alone_satisfies_responsible_person() {
        let mut i = base();
        i.has_officer_director_or_designated_status = true;
        let r = check(&i);
        assert!(r.responsible_person_status);
    }

    #[test]
    fn check_signing_authority_alone_satisfies_responsible_person() {
        let mut i = base();
        i.has_check_signing_or_payment_authority = true;
        let r = check(&i);
        assert!(r.responsible_person_status);
    }

    #[test]
    fn knowledge_alone_satisfies_willfulness() {
        let mut i = base();
        i.significant_control_over_company_finances = true;
        i.knew_trust_fund_taxes_were_due = true;
        let r = check(&i);
        assert!(r.willfulness_engaged);
        assert!(r.tfrp_imposable);
    }

    #[test]
    fn reckless_disregard_alone_satisfies_willfulness() {
        let mut i = base();
        i.significant_control_over_company_finances = true;
        i.recklessly_disregarded_obvious_risk = true;
        let r = check(&i);
        assert!(r.willfulness_engaged);
    }

    #[test]
    fn preferential_creditor_payment_alone_satisfies_willfulness() {
        let mut i = base();
        i.significant_control_over_company_finances = true;
        i.used_funds_to_pay_other_creditors = true;
        let r = check(&i);
        assert!(r.willfulness_engaged);
    }

    #[test]
    fn responsible_person_without_willfulness_no_tfrp() {
        let mut i = base();
        i.significant_control_over_company_finances = true;
        let r = check(&i);
        assert!(r.responsible_person_status);
        assert!(!r.willfulness_engaged);
        assert!(!r.tfrp_imposable);
    }

    #[test]
    fn willfulness_without_responsible_person_no_tfrp() {
        let mut i = base();
        i.knew_trust_fund_taxes_were_due = true;
        let r = check(&i);
        assert!(!r.responsible_person_status);
        assert!(r.willfulness_engaged);
        assert!(!r.tfrp_imposable);
    }

    #[test]
    fn preliminary_notice_compliance_when_60_day_window_provided() {
        let r = check(&full_tfrp());
        assert!(r.preliminary_notice_compliance);
        assert!(!r.preliminary_notice_violation);
    }

    #[test]
    fn preliminary_notice_violation_when_window_not_provided() {
        let mut i = full_tfrp();
        i.preliminary_notice_60_day_window_provided = false;
        let r = check(&i);
        assert!(!r.preliminary_notice_compliance);
        assert!(r.preliminary_notice_violation);
        assert!(r.notes.iter().any(|n| n.contains("§ 6672(b)(1)")
            && n.contains("Letter 1153")
            && n.contains("Form 2751")));
    }

    #[test]
    fn preliminary_notice_violation_only_engaged_when_tfrp_imposable() {
        let mut i = base();
        i.preliminary_notice_60_day_window_provided = false;
        let r = check(&i);
        assert!(!r.preliminary_notice_violation);
    }

    #[test]
    fn contribution_right_engaged_with_multiple_responsible_persons() {
        let mut i = full_tfrp();
        i.multiple_responsible_persons_involved = true;
        let r = check(&i);
        assert!(r.contribution_right_engaged);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6672(d)") && n.contains("JOINTLY AND SEVERALLY")));
    }

    #[test]
    fn contribution_right_not_engaged_without_multiple_persons() {
        let r = check(&full_tfrp());
        assert!(!r.contribution_right_engaged);
    }

    #[test]
    fn nondischargeable_in_bankruptcy_always_true() {
        let r1 = check(&base());
        let r2 = check(&full_tfrp());
        assert!(r1.nondischargeable_in_bankruptcy);
        assert!(r2.nondischargeable_in_bankruptcy);
    }

    #[test]
    fn bankruptcy_attempt_surfaces_nondischargeable_note() {
        let mut i = full_tfrp();
        i.bankruptcy_discharge_attempted = true;
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 523(a)(7)") && n.contains("NONDISCHARGEABLE")));
    }

    #[test]
    fn no_bankruptcy_attempt_no_nondischargeable_note() {
        let r = check(&full_tfrp());
        assert!(!r
            .notes
            .iter()
            .any(|n| n.contains("§ 523(a)(7)") && n.contains("NONDISCHARGEABLE")));
    }

    #[test]
    fn citation_pins_subsections_and_supporting_sections() {
        let r = check(&base());
        assert!(r.citation.contains("§§ 6672(a), 6672(b)(1), 6672(d)"));
        assert!(r.citation.contains("3402"));
        assert!(r.citation.contains("3101"));
        assert!(r.citation.contains("3111"));
        assert!(r.citation.contains("3301"));
        assert!(r.citation.contains("IRM 8.25.1"));
        assert!(r.citation.contains("§§ 523(a)(7), 507(a)(8)(C)"));
    }

    #[test]
    fn trust_fund_taxes_note_lists_3402_and_3101() {
        let r = check(&base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 3402") && n.contains("§ 3101")));
    }

    #[test]
    fn non_trust_fund_note_lists_3111_and_3301() {
        let r = check(&base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 3111") && n.contains("§ 3301")));
    }

    #[test]
    fn tfrp_imposable_note_describes_100_percent_personal() {
        let r = check(&full_tfrp());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6672 TFRP IMPOSABLE") && n.contains("100%")));
    }

    #[test]
    fn responsible_person_satisfied_note_lists_three_factors() {
        let mut i = base();
        i.significant_control_over_company_finances = true;
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("responsible person prong satisfied")
                && n.contains("status")
                && n.contains("authority")));
    }

    #[test]
    fn responsible_person_not_satisfied_note_when_no_factors() {
        let r = check(&base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("responsible person prong NOT satisfied")));
    }

    #[test]
    fn willfulness_satisfied_note_describes_preferential_creditor() {
        let mut i = base();
        i.significant_control_over_company_finances = true;
        i.used_funds_to_pay_other_creditors = true;
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("willfulness prong satisfied") && n.contains("OTHER creditors")));
    }

    #[test]
    fn willfulness_no_evil_intent_required_note() {
        let mut i = base();
        i.significant_control_over_company_finances = true;
        i.knew_trust_fund_taxes_were_due = true;
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("no evil intent required")));
    }

    #[test]
    fn irm_8_25_1_note_always_present() {
        let r = check(&base());
        assert!(r.notes.iter().any(|n| n.contains("IRM 8.25.1")));
    }

    #[test]
    fn responsible_person_three_factor_disjunctive_truth_table() {
        let combinations: [(bool, bool, bool); 8] = [
            (false, false, false),
            (true, false, false),
            (false, true, false),
            (false, false, true),
            (true, true, false),
            (true, false, true),
            (false, true, true),
            (true, true, true),
        ];

        for (control, status, authority) in combinations {
            let mut i = base();
            i.significant_control_over_company_finances = control;
            i.has_officer_director_or_designated_status = status;
            i.has_check_signing_or_payment_authority = authority;
            let r = check(&i);
            let expected = control || status || authority;
            assert_eq!(r.responsible_person_status, expected);
        }
    }

    #[test]
    fn willfulness_three_factor_disjunctive_truth_table() {
        let combinations: [(bool, bool, bool); 8] = [
            (false, false, false),
            (true, false, false),
            (false, true, false),
            (false, false, true),
            (true, true, false),
            (true, false, true),
            (false, true, true),
            (true, true, true),
        ];

        for (know, reckless, preferential) in combinations {
            let mut i = base();
            i.knew_trust_fund_taxes_were_due = know;
            i.recklessly_disregarded_obvious_risk = reckless;
            i.used_funds_to_pay_other_creditors = preferential;
            let r = check(&i);
            let expected = know || reckless || preferential;
            assert_eq!(r.willfulness_engaged, expected);
        }
    }

    #[test]
    fn tfrp_imposable_conjunctive_truth_table() {
        for rp in [false, true] {
            for w in [false, true] {
                let mut i = base();
                if rp {
                    i.significant_control_over_company_finances = true;
                }
                if w {
                    i.knew_trust_fund_taxes_were_due = true;
                }
                let r = check(&i);
                assert_eq!(r.tfrp_imposable, rp && w);
            }
        }
    }

    #[test]
    fn willfulness_negative_note_when_no_factors() {
        let mut i = base();
        i.significant_control_over_company_finances = true;
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("willfulness prong NOT satisfied")));
    }

    #[test]
    fn preliminary_notice_compliance_note_when_satisfied_and_tfrp_imposable() {
        let r = check(&full_tfrp());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6672(b)(1)") && n.contains("60-day waiting period satisfied")));
    }
}
