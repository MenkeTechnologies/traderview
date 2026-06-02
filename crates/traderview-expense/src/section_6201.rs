//! IRC § 6201 — Assessment authority. Foundational grant of
//! IRS power to determine and assess tax liability.
//! Procedural predicate for § 6203 (method of assessment),
//! § 6303 (notice and demand), § 6321 (lien attachment),
//! § 6331 (levy authority). Trader-critical because § 6201(d)
//! shifts burden of proof to Secretary in deficiency
//! proceedings where taxpayer disputes a third-party
//! information return (1099-B broker reporting, 1099-K
//! payment-card processor reporting, K-1 partnership
//! reporting) AND has fully cooperated with IRS.
//!
//! **§ 6201(a) General rule** — Secretary is authorized and
//! required to make inquiries, determinations, and
//! assessments of all taxes (including interest, additional
//! amounts, additions to the tax, and assessable penalties)
//! imposed by Title 26, which have not been duly paid by
//! stamp at the time and manner provided by law:
//! 1. **§ 6201(a)(1) Taxes shown on return** — Secretary
//!    shall assess all taxes determined by the taxpayer or
//!    Secretary as to which returns or lists are made.
//! 2. **§ 6201(a)(2) Unpaid taxes payable by stamp** —
//!    Secretary shall establish by regulations the mode and
//!    time for the collection of unpaid taxes payable by
//!    stamp.
//! 3. **§ 6201(a)(3) Erroneous income tax prepayment
//!    credits** — if a return contains an overstatement of
//!    income-tax-withheld-at-source credit or estimated-tax
//!    credit, the overstated amount may be assessed by
//!    Secretary in the same manner as a mathematical or
//!    clerical error appearing on the return, EXCEPT that
//!    § 6213(b)(2) abatement-of-math-error procedures DO
//!    NOT apply to § 6201(a)(3) assessments.
//!
//! **§ 6201(b) Amount not to be assessed** — exception:
//! assessment of amount of deficiency restricted by § 6213
//! (taxpayer entitled to deficiency-procedure protections
//! before such assessment may be made).
//!
//! **§ 6201(c) Compensation of child** — any income tax under
//! subtitle A attributable to compensation for personal
//! services rendered by a child may be assessed against the
//! child or the parent who must include the income.
//!
//! **§ 6201(d) Required reasonable verification of
//! information returns** — in any court proceeding, if a
//! taxpayer **asserts a reasonable dispute** with respect to
//! any item of income reported on an information return
//! filed with the Secretary under subpart B or C of part III
//! of subchapter A of chapter 61 by a third party AND the
//! taxpayer **has fully cooperated with the Secretary**
//! (including providing access to and inspection of all
//! witnesses, information, and documents within the control
//! of the taxpayer as reasonably requested), the Secretary
//! shall **have the burden of producing reasonable and
//! probative information** concerning such deficiency in
//! addition to such information return.
//!
//! **§ 6201(e) Deficiency proceedings** — cross-reference to
//! § 6211 et seq. for deficiency procedures.
//!
//! Citations: 26 USC § 6201(a)-(d); 26 CFR § 301.6201-1;
//! § 6203 (method of assessment); § 6212 (SNOD); § 6213
//! (deficiency procedures); § 6303 (notice and demand);
//! § 6321 (lien); § 6331 (levy); § 6020(b) (substitute
//! returns); chapter 61 subpart B + C (information returns);
//! Internal Revenue Service Restructuring and Reform Act of
//! 1998 (RRA 98 § 3201) — added § 6201(d).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AssessmentBasis {
    /// § 6201(a)(1) — taxes shown on return.
    TaxesShownOnReturn,
    /// § 6201(a)(2) — unpaid taxes payable by stamp.
    UnpaidStampTaxes,
    /// § 6201(a)(3) — erroneous income tax prepayment credits
    /// (overstated withholding or estimated tax).
    ErroneousPrepaymentCredits,
    /// § 6201(c) — child's personal-services income assessed
    /// against parent or child.
    ChildPersonalServicesIncome,
    /// § 6020(b) substitute-for-return assessment.
    SubstituteForReturn,
    /// No statutory authority cited.
    NoAuthority,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6201Input {
    pub assessment_basis: AssessmentBasis,
    /// Whether assessment is restricted under § 6213(a)
    /// (deficiency notice + 90-day Tax Court window required
    /// before assessment).
    pub deficiency_restriction_applies: bool,
    /// Whether SNOD under § 6212 was issued before this
    /// assessment.
    pub snod_issued: bool,
    /// Whether 90-day petition window has expired.
    pub petition_window_expired: bool,
    /// Whether taxpayer disputes a third-party information
    /// return (1099-B / 1099-K / K-1) item in a court
    /// proceeding.
    pub asserts_information_return_dispute: bool,
    /// Whether taxpayer's dispute is reasonable (asserts
    /// non-frivolous factual or legal basis).
    pub dispute_is_reasonable: bool,
    /// Whether taxpayer has fully cooperated with Secretary
    /// (provided access to witnesses, information, documents
    /// reasonably requested).
    pub taxpayer_fully_cooperated: bool,
    /// Whether Secretary has produced reasonable and probative
    /// information beyond just the information return.
    pub secretary_produced_additional_probative_evidence: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6201Result {
    pub assessment_authority_engaged: bool,
    pub assessment_may_proceed: bool,
    pub deficiency_restriction_satisfied: bool,
    pub burden_of_proof_shifted_to_secretary: bool,
    pub secretary_satisfied_additional_evidence_burden: bool,
    pub math_error_abatement_unavailable_for_a3: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section6201Input) -> Section6201Result {
    let mut failure_reasons: Vec<String> = Vec::new();

    let assessment_authority_engaged = !matches!(input.assessment_basis, AssessmentBasis::NoAuthority);

    if !assessment_authority_engaged {
        failure_reasons.push(
            "26 USC § 6201(a) — assessment must be supported by statutory authority (return-shown taxes, stamp taxes, erroneous prepayment credits, child compensation, or § 6020(b) substitute return)".to_string(),
        );
    }

    let deficiency_satisfied = if input.deficiency_restriction_applies {
        input.snod_issued && input.petition_window_expired
    } else {
        true
    };

    if input.deficiency_restriction_applies && !deficiency_satisfied {
        failure_reasons.push(
            "26 USC § 6201(b) + § 6213(a) — assessment restricted until § 6212 SNOD issued AND 90-day Tax Court petition window expired (150 days for taxpayers outside US)".to_string(),
        );
    }

    let burden_shifted = input.asserts_information_return_dispute
        && input.dispute_is_reasonable
        && input.taxpayer_fully_cooperated;

    let secretary_satisfied_burden = !burden_shifted
        || input.secretary_produced_additional_probative_evidence;

    if burden_shifted && !secretary_satisfied_burden {
        failure_reasons.push(
            "26 USC § 6201(d) — Secretary failed to produce reasonable and probative information beyond third-party information return when taxpayer asserted reasonable dispute and fully cooperated".to_string(),
        );
    }

    let math_error_abatement_unavailable_for_a3 =
        matches!(input.assessment_basis, AssessmentBasis::ErroneousPrepaymentCredits);

    let may_proceed = assessment_authority_engaged
        && deficiency_satisfied
        && secretary_satisfied_burden;

    let notes: Vec<String> = vec![
        "26 USC § 6201(a) — Secretary authorized and required to make inquiries, determinations, and assessments of all taxes imposed by Title 26 including interest, additional amounts, additions to tax, and assessable penalties".to_string(),
        "26 USC § 6201(a)(1) — taxes shown on return assessed by Secretary; § 6201(a)(2) — stamp taxes; § 6201(a)(3) — erroneous prepayment credits (overstated withholding or estimated tax) assessed as math/clerical error WITHOUT § 6213(b)(2) abatement availability".to_string(),
        "26 USC § 6201(b) + § 6213(a) — assessment of deficiency restricted until § 6212 SNOD issued AND 90-day Tax Court petition window (150-day window for taxpayers outside US) has expired".to_string(),
        "26 USC § 6201(c) — child's compensation for personal services may be assessed against parent who must include income".to_string(),
        "26 USC § 6201(d) (RRA 98 § 3201) — burden-shifting rule: in court proceeding where taxpayer asserts reasonable dispute regarding information return item AND fully cooperated, Secretary bears burden of producing reasonable and probative information beyond information return itself; trader-critical for 1099-B + 1099-K + K-1 disputes".to_string(),
        "Cross-references: § 6201 is foundational authority predicate for § 6203 (method of assessment) + § 6303 (notice and demand) + § 6321 (lien) + § 6331 (levy); § 6020(b) substitute-for-return power independent statutory authority".to_string(),
    ];

    Section6201Result {
        assessment_authority_engaged,
        assessment_may_proceed: may_proceed,
        deficiency_restriction_satisfied: deficiency_satisfied,
        burden_of_proof_shifted_to_secretary: burden_shifted,
        secretary_satisfied_additional_evidence_burden: secretary_satisfied_burden,
        math_error_abatement_unavailable_for_a3,
        failure_reasons,
        citation: "26 USC § 6201(a)-(d); 26 CFR § 301.6201-1; § 6203; § 6212; § 6213; § 6303; § 6321; § 6331; § 6020(b); RRA 98 § 3201",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_base() -> Section6201Input {
        Section6201Input {
            assessment_basis: AssessmentBasis::TaxesShownOnReturn,
            deficiency_restriction_applies: false,
            snod_issued: false,
            petition_window_expired: false,
            asserts_information_return_dispute: false,
            dispute_is_reasonable: false,
            taxpayer_fully_cooperated: false,
            secretary_produced_additional_probative_evidence: false,
        }
    }

    #[test]
    fn taxes_shown_on_return_assessment_proceeds() {
        let r = check(&valid_base());
        assert!(r.assessment_authority_engaged);
        assert!(r.assessment_may_proceed);
    }

    #[test]
    fn no_statutory_authority_fails() {
        let mut i = valid_base();
        i.assessment_basis = AssessmentBasis::NoAuthority;
        let r = check(&i);
        assert!(!r.assessment_authority_engaged);
        assert!(!r.assessment_may_proceed);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6201(a)") && f.contains("statutory authority")));
    }

    #[test]
    fn unpaid_stamp_taxes_authority_engaged() {
        let mut i = valid_base();
        i.assessment_basis = AssessmentBasis::UnpaidStampTaxes;
        let r = check(&i);
        assert!(r.assessment_authority_engaged);
        assert!(r.assessment_may_proceed);
    }

    #[test]
    fn erroneous_prepayment_credits_authority_engaged() {
        let mut i = valid_base();
        i.assessment_basis = AssessmentBasis::ErroneousPrepaymentCredits;
        let r = check(&i);
        assert!(r.assessment_authority_engaged);
        assert!(r.math_error_abatement_unavailable_for_a3);
    }

    #[test]
    fn child_personal_services_income_authority_engaged() {
        let mut i = valid_base();
        i.assessment_basis = AssessmentBasis::ChildPersonalServicesIncome;
        let r = check(&i);
        assert!(r.assessment_authority_engaged);
        assert!(r.assessment_may_proceed);
    }

    #[test]
    fn substitute_for_return_authority_engaged() {
        let mut i = valid_base();
        i.assessment_basis = AssessmentBasis::SubstituteForReturn;
        let r = check(&i);
        assert!(r.assessment_authority_engaged);
    }

    #[test]
    fn deficiency_restriction_without_snod_blocks_assessment() {
        let mut i = valid_base();
        i.deficiency_restriction_applies = true;
        i.snod_issued = false;
        let r = check(&i);
        assert!(!r.deficiency_restriction_satisfied);
        assert!(!r.assessment_may_proceed);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6201(b)") && f.contains("§ 6213(a)")));
    }

    #[test]
    fn deficiency_restriction_with_snod_and_expired_window_satisfied() {
        let mut i = valid_base();
        i.deficiency_restriction_applies = true;
        i.snod_issued = true;
        i.petition_window_expired = true;
        let r = check(&i);
        assert!(r.deficiency_restriction_satisfied);
        assert!(r.assessment_may_proceed);
    }

    #[test]
    fn deficiency_restriction_with_snod_but_window_still_open_blocks() {
        let mut i = valid_base();
        i.deficiency_restriction_applies = true;
        i.snod_issued = true;
        i.petition_window_expired = false;
        let r = check(&i);
        assert!(!r.deficiency_restriction_satisfied);
        assert!(!r.assessment_may_proceed);
    }

    #[test]
    fn deficiency_restriction_inapplicable_proceeds() {
        let mut i = valid_base();
        i.deficiency_restriction_applies = false;
        i.snod_issued = false;
        let r = check(&i);
        assert!(r.deficiency_restriction_satisfied);
        assert!(r.assessment_may_proceed);
    }

    #[test]
    fn burden_shift_requires_all_three_conditions() {
        let mut i = valid_base();
        i.asserts_information_return_dispute = true;
        i.dispute_is_reasonable = true;
        i.taxpayer_fully_cooperated = true;
        let r = check(&i);
        assert!(r.burden_of_proof_shifted_to_secretary);
    }

    #[test]
    fn burden_shift_fails_without_dispute_assertion() {
        let mut i = valid_base();
        i.asserts_information_return_dispute = false;
        i.dispute_is_reasonable = true;
        i.taxpayer_fully_cooperated = true;
        let r = check(&i);
        assert!(!r.burden_of_proof_shifted_to_secretary);
    }

    #[test]
    fn burden_shift_fails_without_reasonable_dispute() {
        let mut i = valid_base();
        i.asserts_information_return_dispute = true;
        i.dispute_is_reasonable = false;
        i.taxpayer_fully_cooperated = true;
        let r = check(&i);
        assert!(!r.burden_of_proof_shifted_to_secretary);
    }

    #[test]
    fn burden_shift_fails_without_full_cooperation() {
        let mut i = valid_base();
        i.asserts_information_return_dispute = true;
        i.dispute_is_reasonable = true;
        i.taxpayer_fully_cooperated = false;
        let r = check(&i);
        assert!(!r.burden_of_proof_shifted_to_secretary);
    }

    #[test]
    fn burden_shifted_secretary_failed_to_produce_evidence_assessment_fails() {
        let mut i = valid_base();
        i.asserts_information_return_dispute = true;
        i.dispute_is_reasonable = true;
        i.taxpayer_fully_cooperated = true;
        i.secretary_produced_additional_probative_evidence = false;
        let r = check(&i);
        assert!(r.burden_of_proof_shifted_to_secretary);
        assert!(!r.secretary_satisfied_additional_evidence_burden);
        assert!(!r.assessment_may_proceed);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6201(d)")
                && f.contains("reasonable and probative")));
    }

    #[test]
    fn burden_shifted_secretary_produced_evidence_assessment_proceeds() {
        let mut i = valid_base();
        i.asserts_information_return_dispute = true;
        i.dispute_is_reasonable = true;
        i.taxpayer_fully_cooperated = true;
        i.secretary_produced_additional_probative_evidence = true;
        let r = check(&i);
        assert!(r.burden_of_proof_shifted_to_secretary);
        assert!(r.secretary_satisfied_additional_evidence_burden);
        assert!(r.assessment_may_proceed);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&valid_base());
        assert!(r.citation.contains("§ 6201(a)-(d)"));
        assert!(r.citation.contains("26 CFR § 301.6201-1"));
        assert!(r.citation.contains("§ 6203"));
        assert!(r.citation.contains("§ 6212"));
        assert!(r.citation.contains("§ 6213"));
        assert!(r.citation.contains("§ 6303"));
        assert!(r.citation.contains("§ 6321"));
        assert!(r.citation.contains("§ 6331"));
        assert!(r.citation.contains("§ 6020(b)"));
        assert!(r.citation.contains("RRA 98 § 3201"));
    }

    #[test]
    fn note_pins_general_assessment_authority() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6201(a)")
            && n.contains("assessable penalties")
            && n.contains("interest")));
    }

    #[test]
    fn note_pins_three_subsections_a() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6201(a)(1)")
            && n.contains("§ 6201(a)(2)")
            && n.contains("§ 6201(a)(3)")));
    }

    #[test]
    fn note_pins_a3_math_error_unavailable() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6201(a)(3)")
            && n.contains("§ 6213(b)(2) abatement")));
    }

    #[test]
    fn note_pins_b_deficiency_restriction_with_90_150_day() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6201(b)")
            && n.contains("§ 6212")
            && n.contains("90-day")
            && n.contains("150-day")));
    }

    #[test]
    fn note_pins_c_child_compensation() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6201(c)")
            && n.contains("child")));
    }

    #[test]
    fn note_pins_d_burden_shift_rra_98() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6201(d)")
            && n.contains("RRA 98 § 3201")
            && n.contains("1099-B")
            && n.contains("1099-K")
            && n.contains("K-1")));
    }

    #[test]
    fn note_pins_predicate_for_collection_constellation() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6203")
            && n.contains("§ 6303")
            && n.contains("§ 6321")
            && n.contains("§ 6331")));
    }

    #[test]
    fn assessment_basis_truth_table() {
        for (basis, exp_engaged) in [
            (AssessmentBasis::TaxesShownOnReturn, true),
            (AssessmentBasis::UnpaidStampTaxes, true),
            (AssessmentBasis::ErroneousPrepaymentCredits, true),
            (AssessmentBasis::ChildPersonalServicesIncome, true),
            (AssessmentBasis::SubstituteForReturn, true),
            (AssessmentBasis::NoAuthority, false),
        ] {
            let mut i = valid_base();
            i.assessment_basis = basis;
            let r = check(&i);
            assert_eq!(r.assessment_authority_engaged, exp_engaged);
        }
    }

    #[test]
    fn deficiency_restriction_truth_table() {
        for (applies, snod, expired, exp_satisfied) in [
            (false, false, false, true),
            (false, true, true, true),
            (true, false, false, false),
            (true, false, true, false),
            (true, true, false, false),
            (true, true, true, true),
        ] {
            let mut i = valid_base();
            i.deficiency_restriction_applies = applies;
            i.snod_issued = snod;
            i.petition_window_expired = expired;
            let r = check(&i);
            assert_eq!(
                r.deficiency_restriction_satisfied, exp_satisfied,
                "applies={} snod={} expired={}",
                applies, snod, expired
            );
        }
    }

    #[test]
    fn burden_shift_truth_table_eight_cells() {
        for (dispute, reasonable, cooperated, exp_shifted) in [
            (false, false, false, false),
            (false, false, true, false),
            (false, true, false, false),
            (false, true, true, false),
            (true, false, false, false),
            (true, false, true, false),
            (true, true, false, false),
            (true, true, true, true),
        ] {
            let mut i = valid_base();
            i.asserts_information_return_dispute = dispute;
            i.dispute_is_reasonable = reasonable;
            i.taxpayer_fully_cooperated = cooperated;
            let r = check(&i);
            assert_eq!(
                r.burden_of_proof_shifted_to_secretary, exp_shifted,
                "dispute={} reasonable={} cooperated={}",
                dispute, reasonable, cooperated
            );
        }
    }

    #[test]
    fn a3_uniquely_pins_math_error_abatement_unavailability() {
        for basis in [
            AssessmentBasis::TaxesShownOnReturn,
            AssessmentBasis::UnpaidStampTaxes,
            AssessmentBasis::ChildPersonalServicesIncome,
            AssessmentBasis::SubstituteForReturn,
        ] {
            let mut i = valid_base();
            i.assessment_basis = basis;
            let r = check(&i);
            assert!(!r.math_error_abatement_unavailable_for_a3);
        }

        let mut i_a3 = valid_base();
        i_a3.assessment_basis = AssessmentBasis::ErroneousPrepaymentCredits;
        let r_a3 = check(&i_a3);
        assert!(r_a3.math_error_abatement_unavailable_for_a3);
    }

    #[test]
    fn multiple_failures_stack() {
        let mut i = valid_base();
        i.assessment_basis = AssessmentBasis::NoAuthority;
        i.deficiency_restriction_applies = true;
        i.snod_issued = false;
        i.asserts_information_return_dispute = true;
        i.dispute_is_reasonable = true;
        i.taxpayer_fully_cooperated = true;
        i.secretary_produced_additional_probative_evidence = false;
        let r = check(&i);
        assert_eq!(r.failure_reasons.len(), 3);
        assert!(!r.assessment_may_proceed);
    }

    #[test]
    fn invariant_burden_shift_requires_court_proceeding_context() {
        let mut i = valid_base();
        i.dispute_is_reasonable = true;
        i.taxpayer_fully_cooperated = true;
        i.asserts_information_return_dispute = false;
        let r = check(&i);
        assert!(!r.burden_of_proof_shifted_to_secretary);
    }

    #[test]
    fn invariant_a3_overstated_prepayment_no_a3_abatement_pathway() {
        let mut i = valid_base();
        i.assessment_basis = AssessmentBasis::ErroneousPrepaymentCredits;
        let r = check(&i);
        assert!(r.math_error_abatement_unavailable_for_a3);
        assert!(r.notes.iter().any(|n| n.contains("§ 6213(b)(2) abatement")
            && n.contains("WITHOUT")));
    }
}
