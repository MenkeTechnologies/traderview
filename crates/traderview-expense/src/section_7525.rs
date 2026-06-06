//! IRC § 7525 — federally authorized tax practitioner (FATP)
//! confidentiality privilege. Trader-relevant: traders work
//! with CPAs/EAs for mark-to-market elections (§ 475(f)),
//! straddle (§ 1092) advice, § 1256 60/40 reporting, § 1091
//! wash sale planning, qualified-trader status disputes,
//! international (§ 988, § 1297, § 6038D) advice. Knowing
//! when the practitioner privilege protects communications
//! (and when it categorically does NOT) is operational risk.
//!
//! **§ 7525(a)(1) general rule** — same common-law
//! protections that apply between taxpayer + attorney apply
//! between taxpayer + FATP to the extent the communication
//! would be privileged if between taxpayer + attorney.
//!
//! **§ 7525(a)(3)(A) noncriminal-only limitation** — the
//! privilege may only be asserted in:
//! 1. a noncriminal tax matter before the IRS, or
//! 2. a noncriminal tax proceeding in federal court brought
//!    by or against the United States.
//!
//! Categorical criminal exclusion: any criminal tax matter
//! (grand jury, indictment, criminal-investigation referral)
//! totally voids the privilege.
//!
//! **§ 7525(a)(2) FATP definition** — any individual
//! authorized under federal law to practice before the IRS,
//! subject to federal regulation under 31 USC § 330
//! (Circular 230). Encompasses CPAs, enrolled agents,
//! attorneys (admitted to practice before IRS), enrolled
//! actuaries, and enrolled retirement plan agents.
//!
//! **§ 7525(b) tax-shelter promotion exception** — the
//! privilege does NOT apply to any written communication
//! between an FATP and a person in connection with the
//! promotion of the direct or indirect participation of
//! that person in any tax shelter (as defined in
//! § 6662(d)(2)(C)(ii) — any partnership or other entity,
//! investment plan, or arrangement a significant purpose of
//! which is the avoidance or evasion of federal income tax).
//!
//! **Established judicial limitations (United States v.
//! Frederick, 182 F.3d 496 (7th Cir. 1999) + progeny)** —
//! the privilege does NOT cover:
//! - tax-return preparation work (ministerial, not advice)
//! - state or local tax matters (federal-only)
//! - communications with non-FATP individuals (paralegals,
//!   bookkeepers, financial planners without § 330
//!   authorization)
//!
//! Citations: 26 USC § 7525(a)(1), (a)(2), (a)(3)(A), (b);
//! IRC § 6662(d)(2)(C)(ii) (tax shelter cross-reference);
//! 31 USC § 330 (Circular 230 / FATP authorization);
//! United States v. Frederick, 182 F.3d 496 (7th Cir. 1999)
//! (return prep + criminal limitations).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PractitionerType {
    /// CPA authorized under Circular 230.
    Cpa,
    /// Enrolled agent (EA) authorized under Circular 230.
    EnrolledAgent,
    /// Attorney admitted to practice before IRS.
    Attorney,
    /// Enrolled actuary.
    EnrolledActuary,
    /// Enrolled retirement plan agent.
    EnrolledRetirementPlanAgent,
    /// Bookkeeper / paralegal / financial planner NOT
    /// authorized under § 330 — NOT a FATP.
    NonFatpIndividual,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProceedingContext {
    /// Noncriminal IRS matter (audit, appeals, Tax Court).
    NoncriminalIrsMatter,
    /// Noncriminal federal court (refund suit, summons
    /// enforcement).
    NoncriminalFederalCourt,
    /// Criminal tax matter (grand jury, indictment, IRS-CI
    /// referral) — PRIVILEGE TOTALLY EXCLUDED.
    CriminalTaxMatter,
    /// State / local tax matter — privilege DOES NOT extend.
    StateOrLocalTaxMatter,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CommunicationType {
    /// Oral tax-advice discussion.
    OralTaxAdvice,
    /// Written tax-advice memorandum.
    WrittenTaxAdvice,
    /// Tax-return preparation work (mechanical compilation).
    ReturnPreparationWork,
    /// Written communication promoting tax shelter
    /// participation — § 7525(b) EXCEPTION.
    WrittenTaxShelterPromotion,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section7525Input {
    pub practitioner_type: PractitionerType,
    pub proceeding_context: ProceedingContext,
    pub communication_type: CommunicationType,
    /// Whether the communication would be privileged if
    /// between a taxpayer and an attorney (§ 7525(a)(1)
    /// gating clause).
    pub would_be_privileged_with_attorney: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section7525Result {
    pub privilege_assertable: bool,
    pub fatp_qualified: bool,
    pub noncriminal_context: bool,
    pub federal_tax_matter: bool,
    pub tax_advice_communication: bool,
    pub tax_shelter_promotion_exception: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section7525Input) -> Section7525Result {
    let mut failure_reasons: Vec<String> = Vec::new();

    let fatp_qualified = !matches!(input.practitioner_type, PractitionerType::NonFatpIndividual);
    if !fatp_qualified {
        failure_reasons.push(
            "26 USC § 7525(a)(2) — practitioner not a federally authorized tax practitioner (must be authorized under 31 USC § 330 / Circular 230)"
                .to_string(),
        );
    }

    let noncriminal_context = matches!(
        input.proceeding_context,
        ProceedingContext::NoncriminalIrsMatter | ProceedingContext::NoncriminalFederalCourt
    );
    if matches!(
        input.proceeding_context,
        ProceedingContext::CriminalTaxMatter
    ) {
        failure_reasons.push(
            "26 USC § 7525(a)(3)(A) — privilege categorically EXCLUDED from criminal tax matters (grand jury, indictment, IRS-CI referral)"
                .to_string(),
        );
    }

    let federal_tax_matter = !matches!(
        input.proceeding_context,
        ProceedingContext::StateOrLocalTaxMatter
    );
    if !federal_tax_matter {
        failure_reasons.push(
            "26 USC § 7525(a)(3)(A) — privilege does NOT extend to state or local tax matters (federal-only)"
                .to_string(),
        );
    }

    let tax_advice_communication = matches!(
        input.communication_type,
        CommunicationType::OralTaxAdvice | CommunicationType::WrittenTaxAdvice
    );
    if matches!(
        input.communication_type,
        CommunicationType::ReturnPreparationWork
    ) {
        failure_reasons.push(
            "United States v. Frederick, 182 F.3d 496 (7th Cir. 1999) — privilege does NOT cover tax-return preparation work (ministerial, not advice)"
                .to_string(),
        );
    }

    let tax_shelter_promotion = matches!(
        input.communication_type,
        CommunicationType::WrittenTaxShelterPromotion
    );
    if tax_shelter_promotion {
        failure_reasons.push(
            "26 USC § 7525(b) — privilege does NOT apply to written communications between FATP and person in connection with the promotion of tax shelter participation (§ 6662(d)(2)(C)(ii) cross-reference)"
                .to_string(),
        );
    }

    if !input.would_be_privileged_with_attorney {
        failure_reasons.push(
            "26 USC § 7525(a)(1) — communication would NOT be privileged if between taxpayer and attorney (gating clause)"
                .to_string(),
        );
    }

    let privilege_assertable = failure_reasons.is_empty();

    let notes: Vec<String> = vec![
        "26 USC § 7525(a)(1) — same common-law protections that apply between taxpayer + attorney apply between taxpayer + federally authorized tax practitioner"
            .to_string(),
        "26 USC § 7525(a)(3)(A) — privilege assertable ONLY in noncriminal tax matter before IRS or noncriminal federal court tax proceeding brought by or against the United States"
            .to_string(),
        "26 USC § 7525(b) — written tax-shelter-promotion communications categorically excluded (§ 6662(d)(2)(C)(ii) tax shelter definition)"
            .to_string(),
    ];

    Section7525Result {
        privilege_assertable,
        fatp_qualified,
        noncriminal_context,
        federal_tax_matter,
        tax_advice_communication,
        tax_shelter_promotion_exception: tax_shelter_promotion,
        failure_reasons,
        citation: "26 USC § 7525(a)(1), (a)(2), (a)(3)(A), (b); 31 USC § 330",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn privileged_base() -> Section7525Input {
        Section7525Input {
            practitioner_type: PractitionerType::Cpa,
            proceeding_context: ProceedingContext::NoncriminalIrsMatter,
            communication_type: CommunicationType::OralTaxAdvice,
            would_be_privileged_with_attorney: true,
        }
    }

    #[test]
    fn cpa_noncriminal_oral_advice_privileged() {
        let r = check(&privileged_base());
        assert!(r.privilege_assertable);
        assert!(r.fatp_qualified);
        assert!(r.noncriminal_context);
        assert!(r.federal_tax_matter);
        assert!(r.tax_advice_communication);
        assert!(!r.tax_shelter_promotion_exception);
        assert!(r.failure_reasons.is_empty());
    }

    #[test]
    fn ea_noncriminal_written_advice_privileged() {
        let mut i = privileged_base();
        i.practitioner_type = PractitionerType::EnrolledAgent;
        i.communication_type = CommunicationType::WrittenTaxAdvice;
        let r = check(&i);
        assert!(r.privilege_assertable);
    }

    #[test]
    fn attorney_noncriminal_federal_court_privileged() {
        let mut i = privileged_base();
        i.practitioner_type = PractitionerType::Attorney;
        i.proceeding_context = ProceedingContext::NoncriminalFederalCourt;
        let r = check(&i);
        assert!(r.privilege_assertable);
    }

    #[test]
    fn criminal_tax_matter_categorically_voids_privilege() {
        let mut i = privileged_base();
        i.proceeding_context = ProceedingContext::CriminalTaxMatter;
        let r = check(&i);
        assert!(!r.privilege_assertable);
        assert!(!r.noncriminal_context);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 7525(a)(3)(A)") && f.contains("criminal")));
    }

    #[test]
    fn state_or_local_tax_voids_privilege() {
        let mut i = privileged_base();
        i.proceeding_context = ProceedingContext::StateOrLocalTaxMatter;
        let r = check(&i);
        assert!(!r.privilege_assertable);
        assert!(!r.federal_tax_matter);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("state or local")));
    }

    #[test]
    fn return_preparation_work_excluded() {
        let mut i = privileged_base();
        i.communication_type = CommunicationType::ReturnPreparationWork;
        let r = check(&i);
        assert!(!r.privilege_assertable);
        assert!(!r.tax_advice_communication);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("United States v. Frederick")));
    }

    #[test]
    fn written_tax_shelter_promotion_excluded() {
        let mut i = privileged_base();
        i.communication_type = CommunicationType::WrittenTaxShelterPromotion;
        let r = check(&i);
        assert!(!r.privilege_assertable);
        assert!(r.tax_shelter_promotion_exception);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 7525(b)") && f.contains("§ 6662(d)(2)(C)(ii)")));
    }

    #[test]
    fn non_fatp_individual_voids_privilege() {
        let mut i = privileged_base();
        i.practitioner_type = PractitionerType::NonFatpIndividual;
        let r = check(&i);
        assert!(!r.privilege_assertable);
        assert!(!r.fatp_qualified);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 7525(a)(2)") && f.contains("31 USC § 330")));
    }

    #[test]
    fn would_not_be_attorney_privileged_voids() {
        let mut i = privileged_base();
        i.would_be_privileged_with_attorney = false;
        let r = check(&i);
        assert!(!r.privilege_assertable);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 7525(a)(1)") && f.contains("gating clause")));
    }

    #[test]
    fn multiple_disqualifiers_stack() {
        let i = Section7525Input {
            practitioner_type: PractitionerType::NonFatpIndividual,
            proceeding_context: ProceedingContext::CriminalTaxMatter,
            communication_type: CommunicationType::ReturnPreparationWork,
            would_be_privileged_with_attorney: false,
        };
        let r = check(&i);
        assert!(!r.privilege_assertable);
        assert!(r.failure_reasons.len() >= 4);
    }

    #[test]
    fn fatp_practitioner_types_all_qualify() {
        for pt in [
            PractitionerType::Cpa,
            PractitionerType::EnrolledAgent,
            PractitionerType::Attorney,
            PractitionerType::EnrolledActuary,
            PractitionerType::EnrolledRetirementPlanAgent,
        ] {
            let mut i = privileged_base();
            i.practitioner_type = pt;
            let r = check(&i);
            assert!(r.fatp_qualified);
        }
    }

    #[test]
    fn enrolled_actuary_privileged() {
        let mut i = privileged_base();
        i.practitioner_type = PractitionerType::EnrolledActuary;
        let r = check(&i);
        assert!(r.privilege_assertable);
    }

    #[test]
    fn enrolled_retirement_plan_agent_privileged() {
        let mut i = privileged_base();
        i.practitioner_type = PractitionerType::EnrolledRetirementPlanAgent;
        let r = check(&i);
        assert!(r.privilege_assertable);
    }

    #[test]
    fn citation_pins_all_relevant_subsections() {
        let r = check(&privileged_base());
        assert!(r.citation.contains("§ 7525(a)(1)"));
        assert!(r.citation.contains("(a)(2)"));
        assert!(r.citation.contains("(a)(3)(A)"));
        assert!(r.citation.contains("(b)"));
        assert!(r.citation.contains("31 USC § 330"));
    }

    #[test]
    fn note_pins_attorney_client_extension() {
        let r = check(&privileged_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 7525(a)(1)") && n.contains("attorney")));
    }

    #[test]
    fn note_pins_noncriminal_only_limitation() {
        let r = check(&privileged_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 7525(a)(3)(A)") && n.contains("noncriminal")));
    }

    #[test]
    fn note_pins_tax_shelter_promotion_exception() {
        let r = check(&privileged_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 7525(b)") && n.contains("§ 6662(d)(2)(C)(ii)")));
    }

    #[test]
    fn proceeding_context_truth_table() {
        for (ctx, should_pass) in [
            (ProceedingContext::NoncriminalIrsMatter, true),
            (ProceedingContext::NoncriminalFederalCourt, true),
            (ProceedingContext::CriminalTaxMatter, false),
            (ProceedingContext::StateOrLocalTaxMatter, false),
        ] {
            let mut i = privileged_base();
            i.proceeding_context = ctx;
            let r = check(&i);
            assert_eq!(r.privilege_assertable, should_pass);
        }
    }

    #[test]
    fn communication_type_truth_table() {
        for (ct, should_pass) in [
            (CommunicationType::OralTaxAdvice, true),
            (CommunicationType::WrittenTaxAdvice, true),
            (CommunicationType::ReturnPreparationWork, false),
            (CommunicationType::WrittenTaxShelterPromotion, false),
        ] {
            let mut i = privileged_base();
            i.communication_type = ct;
            let r = check(&i);
            assert_eq!(r.privilege_assertable, should_pass);
        }
    }

    #[test]
    fn criminal_uniquely_categorical_invariant() {
        let mut i_criminal = privileged_base();
        i_criminal.proceeding_context = ProceedingContext::CriminalTaxMatter;
        let r_criminal = check(&i_criminal);
        assert!(!r_criminal.noncriminal_context);
        assert!(r_criminal.federal_tax_matter);

        let mut i_state = privileged_base();
        i_state.proceeding_context = ProceedingContext::StateOrLocalTaxMatter;
        let r_state = check(&i_state);
        assert!(!r_state.federal_tax_matter);
        assert!(!r_state.privilege_assertable);
    }

    #[test]
    fn six_conjunctive_elements_all_required() {
        let r = check(&privileged_base());
        assert!(r.fatp_qualified);
        assert!(r.noncriminal_context);
        assert!(r.federal_tax_matter);
        assert!(r.tax_advice_communication);
        assert!(!r.tax_shelter_promotion_exception);
        assert!(r.privilege_assertable);
    }

    #[test]
    fn shelter_exception_uniquely_written_only_invariant() {
        let mut i_oral = privileged_base();
        i_oral.communication_type = CommunicationType::OralTaxAdvice;
        let r_oral = check(&i_oral);
        assert!(r_oral.privilege_assertable);
        assert!(!r_oral.tax_shelter_promotion_exception);

        let mut i_written_shelter = privileged_base();
        i_written_shelter.communication_type = CommunicationType::WrittenTaxShelterPromotion;
        let r_written_shelter = check(&i_written_shelter);
        assert!(!r_written_shelter.privilege_assertable);
        assert!(r_written_shelter.tax_shelter_promotion_exception);
    }

    #[test]
    fn written_tax_advice_privileged_distinguishes_from_shelter_promotion() {
        let mut i = privileged_base();
        i.communication_type = CommunicationType::WrittenTaxAdvice;
        let r = check(&i);
        assert!(r.privilege_assertable);
        assert!(!r.tax_shelter_promotion_exception);
    }

    #[test]
    fn return_prep_distinguished_from_tax_advice_invariant() {
        let mut i_advice = privileged_base();
        i_advice.communication_type = CommunicationType::OralTaxAdvice;
        let r_advice = check(&i_advice);
        assert!(r_advice.tax_advice_communication);

        let mut i_prep = privileged_base();
        i_prep.communication_type = CommunicationType::ReturnPreparationWork;
        let r_prep = check(&i_prep);
        assert!(!r_prep.tax_advice_communication);
    }

    #[test]
    fn frederick_citation_pinned_for_return_prep_failure() {
        let mut i = privileged_base();
        i.communication_type = CommunicationType::ReturnPreparationWork;
        let r = check(&i);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("182 F.3d 496") && f.contains("7th Cir. 1999")));
    }

    #[test]
    fn three_failure_reasons_stack_for_state_local_non_fatp_return_prep() {
        let i = Section7525Input {
            practitioner_type: PractitionerType::NonFatpIndividual,
            proceeding_context: ProceedingContext::StateOrLocalTaxMatter,
            communication_type: CommunicationType::ReturnPreparationWork,
            would_be_privileged_with_attorney: true,
        };
        let r = check(&i);
        assert_eq!(r.failure_reasons.len(), 3);
        assert!(!r.privilege_assertable);
    }
}
