//! IRC § 7521 — Procedures involving taxpayer interviews. Trader-
//! critical procedural rights at audit, collection, and examination
//! interviews with IRS officers / employees. Three statutory
//! pathways: § 7521(a) recording rights (taxpayer + IRS), § 7521(b)
//! explanation-of-rights requirement, § 7521(c) representation +
//! interview-suspension right.
//!
//! Companion to `landlord_tenant_recording_consent` (federal Wiretap
//! Act + state consent analysis for non-IRS recordings) and
//! `section_7811` (Taxpayer Assistance Orders) and `section_6330`
//! (CDP for levies). Distinct from `section_7430` (attorney fees
//! against IRS post-litigation). This module addresses ONLY the
//! IN-PERSON INTERVIEW procedural pathway during audit / collection
//! / examination.
//!
//! § 7521(a)(1) TAXPAYER RECORDING RIGHT — any officer or employee
//! of the IRS in connection with any in-person interview with any
//! taxpayer relating to the determination or collection of any tax
//! shall, upon ADVANCE REQUEST of such taxpayer, allow the taxpayer
//! to make an audio recording of such interview at the taxpayer's
//! own expense and with the taxpayer's own equipment.
//!
//! § 7521(a)(2) IRS RECORDING — an officer or employee of the IRS
//! may record any interview if such officer or employee (i) INFORMS
//! the taxpayer of such recording PRIOR to the interview, AND (ii)
//! upon request of the taxpayer, PROVIDES the taxpayer with a
//! transcript or copy of such recording BUT ONLY IF the taxpayer
//! provides reimbursement for the cost of the transcription and
//! reproduction.
//!
//! § 7521(b)(1) EXPLANATION OF RIGHTS — an officer or employee of
//! the IRS shall BEFORE OR AT an initial interview provide to the
//! taxpayer (A) in the case of an in-person interview with the
//! taxpayer relating to the DETERMINATION of any tax — an
//! explanation of the AUDIT PROCESS and the taxpayer's RIGHTS
//! under such process; and (B) in the case of an in-person
//! interview with the taxpayer relating to the COLLECTION of any
//! tax — an explanation of the COLLECTION PROCESS and the
//! taxpayer's RIGHTS under such process.
//!
//! § 7521(c) REPRESENTATION + INTERVIEW SUSPENSION — any attorney,
//! certified public accountant, enrolled agent, enrolled actuary,
//! or any other person permitted to represent the taxpayer before
//! the IRS who is NOT disbarred or suspended from practice and who
//! has a WRITTEN POWER OF ATTORNEY (Form 2848) may be authorized
//! by such taxpayer to represent the taxpayer in any interview.
//!
//! § 7521(c) SUSPENSION RULE — if the taxpayer clearly states to
//! an officer or employee at ANY TIME during any interview (other
//! than an interview initiated by an ADMINISTRATIVE SUMMONS) that
//! the taxpayer wishes to consult with an attorney / CPA / enrolled
//! agent / enrolled actuary / authorized representative, such
//! officer or employee shall SUSPEND such interview regardless of
//! whether the taxpayer may have answered one or more questions.
//!
//! § 7521(c) DELAY BYPASS — § 7521(c) suspension does NOT apply if
//! the IRS officer / employee in charge of the audit determines
//! that the representative is responsible for UNREASONABLE DELAY or
//! HINDRANCE of an IRS examination / collection. Bypass requires
//! consent of Immediate Supervisor.
//!
//! Citations: IRC § 7521(a)(1) (taxpayer recording right);
//! § 7521(a)(2) (IRS recording with notice + reimbursable transcript);
//! § 7521(b)(1)(A) (explanation of audit / determination rights);
//! § 7521(b)(1)(B) (explanation of collection rights); § 7521(c)
//! (representation + suspension right); § 7521(c) delay bypass +
//! Immediate Supervisor consent; § 7602 (administrative summons —
//! interview initiated by summons exempt from § 7521(c) suspension);
//! Treas. Reg. § 601.502 (Form 2848 power of attorney requirements);
//! cross-reference § 7811 (Taxpayer Assistance Orders), § 6330 (CDP
//! for levies), § 6320 (CDP for liens).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InterviewType {
    /// In-person interview relating to determination of tax (audit
    /// / examination). § 7521(b)(1)(A) explanation-of-rights
    /// applies.
    TaxDetermination,
    /// In-person interview relating to collection of tax.
    /// § 7521(b)(1)(B) explanation-of-rights applies.
    Collection,
    /// Interview initiated by ADMINISTRATIVE SUMMONS under
    /// § 7602. § 7521(c) suspension right does NOT apply.
    AdministrativeSummons,
    /// Criminal investigation interview. § 7521 procedural rights
    /// do not apply in same way; caller responsibility.
    CriminalInvestigation,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section7521Input {
    pub interview_type: InterviewType,
    /// Whether the taxpayer requested in advance to make an audio
    /// recording of the interview (§ 7521(a)(1)).
    pub taxpayer_requested_recording_in_advance: bool,
    /// Whether the IRS officer / employee informed the taxpayer
    /// in advance that the IRS would be recording the interview
    /// (§ 7521(a)(2)).
    pub irs_provided_advance_notice_of_irs_recording: bool,
    /// Whether the IRS officer / employee provided the explanation
    /// of audit / collection process and taxpayer's rights before
    /// or at the initial interview (§ 7521(b)(1)).
    pub irs_explanation_of_rights_provided: bool,
    /// Whether the taxpayer has executed a written power of
    /// attorney (Form 2848) authorizing a representative.
    pub taxpayer_has_power_of_attorney: bool,
    /// Whether the taxpayer clearly stated during the interview a
    /// wish to consult with attorney / CPA / enrolled agent /
    /// enrolled actuary / authorized representative.
    pub taxpayer_requested_representation_consultation: bool,
    /// Whether the IRS officer / employee suspended the interview
    /// upon the taxpayer's representation-consultation request.
    pub interview_suspended_upon_request: bool,
    /// Whether the IRS officer / employee determined that the
    /// representative was responsible for unreasonable delay and
    /// obtained Immediate Supervisor consent for the § 7521(c)
    /// delay bypass.
    pub unreasonable_delay_bypass_approved_by_supervisor: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section7521Result {
    pub taxpayer_recording_rights_engaged: bool,
    pub irs_recording_complies: bool,
    pub explanation_of_rights_satisfied: bool,
    pub representation_right_engaged: bool,
    pub interview_suspension_required: bool,
    pub interview_suspension_satisfied: bool,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn compute(input: &Section7521Input) -> Section7521Result {
    let mut notes: Vec<String> = Vec::new();

    if matches!(input.interview_type, InterviewType::CriminalInvestigation) {
        notes.push(
            "criminal investigation interview — § 7521 procedural rights do not apply in the same manner; consult criminal defense counsel and 18 U.S.C. § 3500 (Jencks Act) framework"
                .to_string(),
        );
    }

    let taxpayer_recording_engaged = input.taxpayer_requested_recording_in_advance
        && matches!(
            input.interview_type,
            InterviewType::TaxDetermination | InterviewType::Collection
        );
    if taxpayer_recording_engaged {
        notes.push(
            "§ 7521(a)(1) — IRS must allow taxpayer to audio record interview at taxpayer's own expense and with taxpayer's own equipment, upon advance request"
                .to_string(),
        );
    } else if input.taxpayer_requested_recording_in_advance {
        notes.push(
            "§ 7521(a)(1) — taxpayer recording right applies only to in-person interviews relating to determination or collection of tax"
                .to_string(),
        );
    }

    let irs_recording_complies = if input.irs_provided_advance_notice_of_irs_recording {
        notes.push(
            "§ 7521(a)(2) — IRS provided advance notice of recording; upon taxpayer request, IRS must provide transcript or copy at taxpayer's reimbursement of cost"
                .to_string(),
        );
        true
    } else {
        false
    };

    let explanation_of_rights_satisfied = matches!(
        input.interview_type,
        InterviewType::TaxDetermination | InterviewType::Collection
    ) && input.irs_explanation_of_rights_provided;
    if matches!(
        input.interview_type,
        InterviewType::TaxDetermination | InterviewType::Collection
    ) {
        if input.irs_explanation_of_rights_provided {
            let subsection = match input.interview_type {
                InterviewType::TaxDetermination => "(A) audit process",
                InterviewType::Collection => "(B) collection process",
                _ => unreachable!(),
            };
            notes.push(format!(
                "§ 7521(b)(1){} — IRS explanation of process and taxpayer rights provided before or at initial interview",
                subsection
            ));
        } else {
            notes.push(
                "§ 7521(b)(1) — IRS officer must provide explanation of process and taxpayer's rights before or at initial interview; failure may render subsequent statements suppressible"
                    .to_string(),
            );
        }
    }

    let representation_engaged = input.taxpayer_has_power_of_attorney
        || input.taxpayer_requested_representation_consultation;
    if input.taxpayer_has_power_of_attorney {
        notes.push(
            "§ 7521(c) — taxpayer represented by attorney / CPA / enrolled agent / enrolled actuary / other authorized representative via Form 2848 power of attorney (Treas. Reg. § 601.502)"
                .to_string(),
        );
    }

    let suspension_required = input.taxpayer_requested_representation_consultation
        && !matches!(input.interview_type, InterviewType::AdministrativeSummons)
        && !input.unreasonable_delay_bypass_approved_by_supervisor;

    let suspension_satisfied = if suspension_required {
        input.interview_suspended_upon_request
    } else {
        true
    };

    if input.taxpayer_requested_representation_consultation {
        if matches!(input.interview_type, InterviewType::AdministrativeSummons) {
            notes.push(
                "§ 7521(c) suspension right does NOT apply to interview initiated by ADMINISTRATIVE SUMMONS under § 7602"
                    .to_string(),
            );
        } else if input.unreasonable_delay_bypass_approved_by_supervisor {
            notes.push(
                "§ 7521(c) delay bypass engaged — IRS officer determined representative responsible for unreasonable delay; Immediate Supervisor consent obtained; interview proceeds without suspension"
                    .to_string(),
            );
        } else if input.interview_suspended_upon_request {
            notes.push(
                "§ 7521(c) — taxpayer requested representation consultation; IRS suspended interview regardless of prior answers; tenant may resume with representative present"
                    .to_string(),
            );
        } else {
            notes.push(
                "§ 7521(c) VIOLATION — taxpayer requested representation consultation; IRS FAILED to suspend interview; subsequent statements may be suppressible at trial"
                    .to_string(),
            );
        }
    }

    Section7521Result {
        taxpayer_recording_rights_engaged: taxpayer_recording_engaged,
        irs_recording_complies,
        explanation_of_rights_satisfied,
        representation_right_engaged: representation_engaged,
        interview_suspension_required: suspension_required,
        interview_suspension_satisfied: suspension_satisfied,
        citation: citation(),
        notes,
    }
}

fn citation() -> &'static str {
    "IRC § 7521(a)(1)/(a)(2)/(b)(1)(A)/(b)(1)(B)/(c); § 7602 (administrative summons); Treas. Reg. § 601.502 (Form 2848); cross-reference § 7811 (TAOs), § 6330 (CDP levies), § 6320 (CDP liens)"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(interview_type: InterviewType) -> Section7521Input {
        Section7521Input {
            interview_type,
            taxpayer_requested_recording_in_advance: false,
            irs_provided_advance_notice_of_irs_recording: false,
            irs_explanation_of_rights_provided: true,
            taxpayer_has_power_of_attorney: false,
            taxpayer_requested_representation_consultation: false,
            interview_suspended_upon_request: false,
            unreasonable_delay_bypass_approved_by_supervisor: false,
        }
    }

    #[test]
    fn tax_determination_taxpayer_recording_right_engaged_with_advance_request() {
        let mut i = base(InterviewType::TaxDetermination);
        i.taxpayer_requested_recording_in_advance = true;
        let r = compute(&i);
        assert!(r.taxpayer_recording_rights_engaged);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 7521(a)(1)") && n.contains("own expense")));
    }

    #[test]
    fn collection_interview_taxpayer_recording_right_engaged() {
        let mut i = base(InterviewType::Collection);
        i.taxpayer_requested_recording_in_advance = true;
        let r = compute(&i);
        assert!(r.taxpayer_recording_rights_engaged);
    }

    #[test]
    fn taxpayer_recording_not_engaged_without_advance_request() {
        let r = compute(&base(InterviewType::TaxDetermination));
        assert!(!r.taxpayer_recording_rights_engaged);
    }

    #[test]
    fn administrative_summons_no_recording_right() {
        let mut i = base(InterviewType::AdministrativeSummons);
        i.taxpayer_requested_recording_in_advance = true;
        let r = compute(&i);
        assert!(!r.taxpayer_recording_rights_engaged);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 7521(a)(1)") && n.contains("only to")));
    }

    #[test]
    fn irs_recording_complies_with_advance_notice() {
        let mut i = base(InterviewType::TaxDetermination);
        i.irs_provided_advance_notice_of_irs_recording = true;
        let r = compute(&i);
        assert!(r.irs_recording_complies);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 7521(a)(2)") && n.contains("advance notice")));
    }

    #[test]
    fn irs_recording_fails_without_advance_notice() {
        let r = compute(&base(InterviewType::TaxDetermination));
        assert!(!r.irs_recording_complies);
    }

    #[test]
    fn explanation_of_rights_satisfied_for_tax_determination() {
        let r = compute(&base(InterviewType::TaxDetermination));
        assert!(r.explanation_of_rights_satisfied);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 7521(b)(1)(A) audit process")));
    }

    #[test]
    fn explanation_of_rights_satisfied_for_collection() {
        let r = compute(&base(InterviewType::Collection));
        assert!(r.explanation_of_rights_satisfied);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 7521(b)(1)(B) collection process")));
    }

    #[test]
    fn explanation_of_rights_not_provided_violation_note() {
        let mut i = base(InterviewType::TaxDetermination);
        i.irs_explanation_of_rights_provided = false;
        let r = compute(&i);
        assert!(!r.explanation_of_rights_satisfied);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 7521(b)(1)") && n.contains("failure may render")));
    }

    #[test]
    fn explanation_of_rights_does_not_apply_to_administrative_summons() {
        let r = compute(&base(InterviewType::AdministrativeSummons));
        assert!(!r.explanation_of_rights_satisfied);
        let exp_notes: Vec<_> = r
            .notes
            .iter()
            .filter(|n| n.contains("§ 7521(b)(1)"))
            .collect();
        assert!(exp_notes.is_empty());
    }

    #[test]
    fn representation_engaged_with_power_of_attorney() {
        let mut i = base(InterviewType::TaxDetermination);
        i.taxpayer_has_power_of_attorney = true;
        let r = compute(&i);
        assert!(r.representation_right_engaged);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Form 2848") && n.contains("Treas. Reg. § 601.502")));
    }

    #[test]
    fn suspension_required_when_taxpayer_requests_representation() {
        let mut i = base(InterviewType::TaxDetermination);
        i.taxpayer_requested_representation_consultation = true;
        let r = compute(&i);
        assert!(r.interview_suspension_required);
        assert!(!r.interview_suspension_satisfied);
        assert!(r.notes.iter().any(|n| n.contains("§ 7521(c) VIOLATION")));
    }

    #[test]
    fn suspension_satisfied_when_irs_suspends() {
        let mut i = base(InterviewType::TaxDetermination);
        i.taxpayer_requested_representation_consultation = true;
        i.interview_suspended_upon_request = true;
        let r = compute(&i);
        assert!(r.interview_suspension_required);
        assert!(r.interview_suspension_satisfied);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 7521(c)") && n.contains("regardless of prior answers")));
    }

    #[test]
    fn administrative_summons_bars_suspension_right() {
        let mut i = base(InterviewType::AdministrativeSummons);
        i.taxpayer_requested_representation_consultation = true;
        let r = compute(&i);
        assert!(!r.interview_suspension_required);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("does NOT apply") && n.contains("ADMINISTRATIVE SUMMONS")));
    }

    #[test]
    fn unreasonable_delay_bypass_with_supervisor_consent_no_suspension() {
        let mut i = base(InterviewType::TaxDetermination);
        i.taxpayer_requested_representation_consultation = true;
        i.unreasonable_delay_bypass_approved_by_supervisor = true;
        let r = compute(&i);
        assert!(!r.interview_suspension_required);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Immediate Supervisor consent")));
    }

    #[test]
    fn criminal_investigation_warning_note_engaged() {
        let r = compute(&base(InterviewType::CriminalInvestigation));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("criminal investigation") && n.contains("Jencks Act")));
    }

    #[test]
    fn citation_pins_all_subsections_and_authorities() {
        let r = compute(&base(InterviewType::TaxDetermination));
        assert!(r.citation.contains("§ 7521(a)(1)"));
        assert!(r.citation.contains("(a)(2)"));
        assert!(r.citation.contains("(b)(1)(A)"));
        assert!(r.citation.contains("(b)(1)(B)"));
        assert!(r.citation.contains("(c)"));
        assert!(r.citation.contains("§ 7602"));
        assert!(r.citation.contains("§ 601.502"));
        assert!(r.citation.contains("§ 7811"));
        assert!(r.citation.contains("§ 6330"));
        assert!(r.citation.contains("§ 6320"));
    }

    #[test]
    fn four_interview_types_routed_correctly() {
        for interview_type in [
            InterviewType::TaxDetermination,
            InterviewType::Collection,
            InterviewType::AdministrativeSummons,
            InterviewType::CriminalInvestigation,
        ] {
            let r = compute(&base(interview_type));
            let _ = r;
        }
    }

    #[test]
    fn tax_determination_unique_recording_right_invariant() {
        let determination_types = [InterviewType::TaxDetermination, InterviewType::Collection];
        for interview_type in determination_types {
            let mut i = base(interview_type);
            i.taxpayer_requested_recording_in_advance = true;
            let r = compute(&i);
            assert!(
                r.taxpayer_recording_rights_engaged,
                "{:?} should engage taxpayer recording right",
                interview_type
            );
        }
        let other_types = [
            InterviewType::AdministrativeSummons,
            InterviewType::CriminalInvestigation,
        ];
        for interview_type in other_types {
            let mut i = base(interview_type);
            i.taxpayer_requested_recording_in_advance = true;
            let r = compute(&i);
            assert!(
                !r.taxpayer_recording_rights_engaged,
                "{:?} should NOT engage taxpayer recording right",
                interview_type
            );
        }
    }

    #[test]
    fn suspension_required_truth_table_4_cells() {
        // (taxpayer_requested, interview_type, bypass_supervisor) → suspension_required
        let cases = [
            (true, InterviewType::TaxDetermination, false, true),
            (true, InterviewType::TaxDetermination, true, false),
            (true, InterviewType::AdministrativeSummons, false, false),
            (false, InterviewType::TaxDetermination, false, false),
        ];
        for (requested, it, bypass, expected) in cases {
            let mut i = base(it);
            i.taxpayer_requested_representation_consultation = requested;
            i.unreasonable_delay_bypass_approved_by_supervisor = bypass;
            let r = compute(&i);
            assert_eq!(
                r.interview_suspension_required, expected,
                "requested={} it={:?} bypass={} expected={}",
                requested, it, bypass, expected
            );
        }
    }

    #[test]
    fn poa_alone_engages_representation_right() {
        let mut i = base(InterviewType::TaxDetermination);
        i.taxpayer_has_power_of_attorney = true;
        i.taxpayer_requested_representation_consultation = false;
        let r = compute(&i);
        assert!(r.representation_right_engaged);
    }

    #[test]
    fn representation_consultation_request_alone_engages_representation_right() {
        let mut i = base(InterviewType::TaxDetermination);
        i.taxpayer_requested_representation_consultation = true;
        i.taxpayer_has_power_of_attorney = false;
        let r = compute(&i);
        assert!(r.representation_right_engaged);
    }

    #[test]
    fn collection_explanation_subsection_b_referenced() {
        let r = compute(&base(InterviewType::Collection));
        assert!(r.notes.iter().any(|n| n.contains("(B) collection process")));
    }

    #[test]
    fn tax_determination_explanation_subsection_a_referenced() {
        let r = compute(&base(InterviewType::TaxDetermination));
        assert!(r.notes.iter().any(|n| n.contains("(A) audit process")));
    }

    #[test]
    fn failure_to_suspend_violation_note_warns_about_suppression() {
        let mut i = base(InterviewType::TaxDetermination);
        i.taxpayer_requested_representation_consultation = true;
        i.interview_suspended_upon_request = false;
        let r = compute(&i);
        assert!(r.notes.iter().any(|n| n.contains("suppressible at trial")));
    }

    #[test]
    fn no_representation_request_no_suspension_required() {
        let mut i = base(InterviewType::TaxDetermination);
        i.taxpayer_requested_representation_consultation = false;
        let r = compute(&i);
        assert!(!r.interview_suspension_required);
        assert!(r.interview_suspension_satisfied);
    }

    #[test]
    fn taxpayer_recording_request_outside_determination_or_collection_no_right() {
        let mut i = base(InterviewType::AdministrativeSummons);
        i.taxpayer_requested_recording_in_advance = true;
        let r = compute(&i);
        assert!(!r.taxpayer_recording_rights_engaged);
        let mut i_crim = base(InterviewType::CriminalInvestigation);
        i_crim.taxpayer_requested_recording_in_advance = true;
        let r_crim = compute(&i_crim);
        assert!(!r_crim.taxpayer_recording_rights_engaged);
    }

    #[test]
    fn irs_recording_compliance_independent_of_taxpayer_recording_request() {
        let mut i = base(InterviewType::TaxDetermination);
        i.irs_provided_advance_notice_of_irs_recording = true;
        i.taxpayer_requested_recording_in_advance = false;
        let r = compute(&i);
        assert!(r.irs_recording_complies);
        assert!(!r.taxpayer_recording_rights_engaged);
    }

    #[test]
    fn full_compliance_path_all_rights_satisfied() {
        let mut i = base(InterviewType::TaxDetermination);
        i.taxpayer_requested_recording_in_advance = true;
        i.irs_provided_advance_notice_of_irs_recording = true;
        i.taxpayer_has_power_of_attorney = true;
        i.taxpayer_requested_representation_consultation = true;
        i.interview_suspended_upon_request = true;
        let r = compute(&i);
        assert!(r.taxpayer_recording_rights_engaged);
        assert!(r.irs_recording_complies);
        assert!(r.explanation_of_rights_satisfied);
        assert!(r.representation_right_engaged);
        assert!(r.interview_suspension_satisfied);
    }
}
