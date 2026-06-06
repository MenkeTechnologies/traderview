//! IRC § 6203 — Method of assessment. The mechanical
//! procedure by which an IRS assessment under § 6201 becomes
//! effective. Trader-procedural-critical because no lawful
//! § 6321 lien attachment OR § 6331 levy authority engages
//! without a valid § 6203 record of assessment. Companion to
//! § 6201 (assessment authority), § 6303 (notice and demand
//! — 60 days after the § 6203 recording), § 6321 (lien
//! arising at moment of assessment + notice and demand),
//! § 6331 (levy authority engages 10 days after notice and
//! demand).
//!
//! **§ 6203 General rule** — the assessment shall be made by
//! **recording the liability of the taxpayer in the office
//! of the Secretary** in accordance with rules or
//! regulations prescribed by the Secretary. **Upon request
//! of the taxpayer, the Secretary shall furnish the
//! taxpayer a copy of the record of assessment.**
//!
//! **26 CFR § 301.6203-1 Method of assessment** — the
//! assessment shall be made by an **assessment officer
//! signing the summary record of assessment**. The summary
//! record, through supporting records, shall provide:
//! 1. **Identification of the taxpayer**;
//! 2. **Character of the liability assessed** (e.g.,
//!    income tax, employment tax, penalty);
//! 3. **Taxable period, if applicable**; AND
//! 4. **Amount of the assessment**.
//!
//! All four elements are mandatory. The date of the
//! assessment is the date the summary record is signed by
//! the assessment officer.
//!
//! **Form 23-C — Assessment Certificate — Summary Record of
//! Assessments** — internal IRS document used to formalize a
//! taxpayer's debt. The creation and signing of this
//! certificate establishes the tax liability in the
//! government's records, providing the legal authority for
//! all subsequent IRS actions (lien, levy, seizure,
//! offset). **Form 23-C is NOT released to taxpayers** —
//! IRS treats it as an internal summary document that does
//! not identify individual taxpayers.
//!
//! **Form 4340 — Certificate of Assessments, Payments, and
//! Other Specified Matters** — the document IRS provides
//! when a taxpayer requests "a copy of the record of
//! assessment" under § 6203. Rev. Rul. 2007-21 confirms IRS
//! may choose among documents containing the required
//! regulatory elements; courts have held Form 4340 is
//! **presumptive evidence of valid assessment**.
//!
//! Citations: 26 USC § 6203; 26 CFR § 301.6203-1; Rev. Rul.
//! 2007-21; Form 23-C (Assessment Certificate); Form 4340
//! (Certificate of Assessments); § 6201 (assessment
//! authority); § 6303 (notice and demand); § 6321 (lien);
//! § 6331 (levy); United States v. Dixon, 672 F. Supp. 503
//! (M.D. Ala. 1987) (signed Form 23-C required); March v.
//! IRS, 335 F.3d 1186 (10th Cir. 2003) (Form 4340 sufficient
//! evidence of assessment).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RecordFormat {
    /// Form 23-C signed paper assessment certificate.
    Form23cSigned,
    /// Computer-generated Form RACS Report 006 / IDRS
    /// summary record (modern equivalent of Form 23-C).
    ComputerGeneratedSummaryRecord,
    /// No summary record of assessment exists.
    NoRecord,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaxpayerCopyDocument {
    /// Form 4340 Certificate of Assessments, Payments, and
    /// Other Specified Matters.
    Form4340,
    /// IMF MFTRA Master File Transcript.
    MasterFileTranscript,
    /// Form 23-C copy (NOT released; would violate IRS
    /// policy).
    Form23cCopy,
    /// No document provided.
    NoneProvided,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6203Input {
    pub record_format: RecordFormat,
    /// Whether assessment officer signed the summary record
    /// (date of assessment = date of signature).
    pub officer_signed: bool,
    /// Whether taxpayer identification appears on summary
    /// record.
    pub taxpayer_identified: bool,
    /// Whether character of liability (income tax /
    /// employment tax / penalty) is specified.
    pub character_of_liability_specified: bool,
    /// Whether taxable period is identified (where
    /// applicable).
    pub taxable_period_identified: bool,
    /// Whether taxable period applies for this assessment
    /// type (e.g., income tax = yes; trust fund recovery
    /// penalty period varies).
    pub taxable_period_applicable: bool,
    /// Whether amount of assessment is specified.
    pub amount_specified: bool,
    /// Whether taxpayer requested copy of record of
    /// assessment.
    pub taxpayer_requested_copy: bool,
    /// Document provided to taxpayer in response.
    pub taxpayer_copy_document: TaxpayerCopyDocument,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6203Result {
    pub assessment_validly_recorded: bool,
    pub officer_signature_present: bool,
    pub all_four_regulatory_elements_present: bool,
    pub taxpayer_identification_present: bool,
    pub character_of_liability_present: bool,
    pub taxable_period_compliant: bool,
    pub amount_present: bool,
    pub taxpayer_copy_request_satisfied: bool,
    pub lien_and_levy_predicate_satisfied: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section6203Input) -> Section6203Result {
    let mut failure_reasons: Vec<String> = Vec::new();

    let record_exists = !matches!(input.record_format, RecordFormat::NoRecord);

    if !record_exists {
        failure_reasons.push(
            "26 USC § 6203 + 26 CFR § 301.6203-1 — assessment must be made by recording liability of taxpayer in office of Secretary; no summary record of assessment exists".to_string(),
        );
    }

    if record_exists && !input.officer_signed {
        failure_reasons.push(
            "26 CFR § 301.6203-1 — assessment shall be made by assessment officer SIGNING the summary record of assessment; date of assessment = date of signature".to_string(),
        );
    }

    if record_exists && !input.taxpayer_identified {
        failure_reasons.push(
            "26 CFR § 301.6203-1 — summary record must provide IDENTIFICATION OF THE TAXPAYER"
                .to_string(),
        );
    }

    if record_exists && !input.character_of_liability_specified {
        failure_reasons.push(
            "26 CFR § 301.6203-1 — summary record must provide CHARACTER OF THE LIABILITY assessed (income tax, employment tax, penalty, etc.)".to_string(),
        );
    }

    let period_compliant = if input.taxable_period_applicable {
        input.taxable_period_identified
    } else {
        true
    };

    if record_exists && input.taxable_period_applicable && !input.taxable_period_identified {
        failure_reasons.push(
            "26 CFR § 301.6203-1 — summary record must identify TAXABLE PERIOD (where applicable)"
                .to_string(),
        );
    }

    if record_exists && !input.amount_specified {
        failure_reasons.push(
            "26 CFR § 301.6203-1 — summary record must specify AMOUNT OF THE ASSESSMENT"
                .to_string(),
        );
    }

    let all_four_elements = input.taxpayer_identified
        && input.character_of_liability_specified
        && period_compliant
        && input.amount_specified;

    let copy_satisfied = if input.taxpayer_requested_copy {
        matches!(
            input.taxpayer_copy_document,
            TaxpayerCopyDocument::Form4340 | TaxpayerCopyDocument::MasterFileTranscript
        )
    } else {
        true
    };

    if input.taxpayer_requested_copy && !copy_satisfied {
        match input.taxpayer_copy_document {
            TaxpayerCopyDocument::Form23cCopy => failure_reasons.push(
                "26 USC § 6203 — IRS treats Form 23-C as internal summary document that does not identify individual taxpayers; Form 23-C copy is NOT released; Form 4340 is the appropriate response per Rev. Rul. 2007-21".to_string(),
            ),
            TaxpayerCopyDocument::NoneProvided => failure_reasons.push(
                "26 USC § 6203 — 'Upon request of the taxpayer, the Secretary SHALL furnish the taxpayer a copy of the record of assessment'; no document provided in response to request".to_string(),
            ),
            _ => {}
        }
    }

    let assessment_valid = record_exists && input.officer_signed && all_four_elements;

    let lien_and_levy_predicate = assessment_valid;

    let notes: Vec<String> = vec![
        "26 USC § 6203 — assessment shall be made by recording liability of taxpayer in office of Secretary in accordance with rules or regulations; upon request of taxpayer, Secretary shall furnish taxpayer copy of record of assessment".to_string(),
        "26 CFR § 301.6203-1 — assessment officer signs the summary record of assessment; summary record (through supporting records) must provide (1) identification of taxpayer, (2) character of liability assessed, (3) taxable period if applicable, and (4) amount of assessment; all four elements mandatory".to_string(),
        "26 CFR § 301.6203-1 — date of assessment is date the summary record is signed by assessment officer; this date triggers § 6303(a) 60-day notice and demand window + § 6502 10-year collection statute".to_string(),
        "Form 23-C Assessment Certificate Summary Record of Assessments — internal IRS document; signed by assessment officer; NOT released to taxpayers (IRS treats as internal summary)".to_string(),
        "Form 4340 Certificate of Assessments Payments and Other Specified Matters — document IRS provides when taxpayer requests § 6203 copy; presumptive evidence of valid assessment in court proceedings (March v. IRS, 335 F.3d 1186)".to_string(),
        "Rev. Rul. 2007-21 — IRS may choose among documents containing items of information listed in 26 CFR § 301.6203-1; not required to provide any particular form".to_string(),
        "Cross-references: § 6203 is procedural predicate for § 6303 notice and demand (60-day window from § 6203 recording) + § 6321 lien (arises at moment of assessment + notice and demand) + § 6331 levy authority (10 days after notice and demand) + § 6502 CSED (10 years from § 6203 recording)".to_string(),
    ];

    Section6203Result {
        assessment_validly_recorded: assessment_valid,
        officer_signature_present: input.officer_signed,
        all_four_regulatory_elements_present: all_four_elements,
        taxpayer_identification_present: input.taxpayer_identified,
        character_of_liability_present: input.character_of_liability_specified,
        taxable_period_compliant: period_compliant,
        amount_present: input.amount_specified,
        taxpayer_copy_request_satisfied: copy_satisfied,
        lien_and_levy_predicate_satisfied: lien_and_levy_predicate,
        failure_reasons,
        citation: "26 USC § 6203; 26 CFR § 301.6203-1; Rev. Rul. 2007-21; Form 23-C; Form 4340; § 6201; § 6303; § 6321; § 6331; § 6502; March v. IRS 335 F.3d 1186 (10th Cir. 2003)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_base() -> Section6203Input {
        Section6203Input {
            record_format: RecordFormat::ComputerGeneratedSummaryRecord,
            officer_signed: true,
            taxpayer_identified: true,
            character_of_liability_specified: true,
            taxable_period_identified: true,
            taxable_period_applicable: true,
            amount_specified: true,
            taxpayer_requested_copy: false,
            taxpayer_copy_document: TaxpayerCopyDocument::NoneProvided,
        }
    }

    #[test]
    fn fully_compliant_assessment_valid() {
        let r = check(&valid_base());
        assert!(r.assessment_validly_recorded);
        assert!(r.all_four_regulatory_elements_present);
        assert!(r.lien_and_levy_predicate_satisfied);
    }

    #[test]
    fn form_23c_signed_compliant() {
        let mut i = valid_base();
        i.record_format = RecordFormat::Form23cSigned;
        let r = check(&i);
        assert!(r.assessment_validly_recorded);
    }

    #[test]
    fn no_record_assessment_invalid() {
        let mut i = valid_base();
        i.record_format = RecordFormat::NoRecord;
        let r = check(&i);
        assert!(!r.assessment_validly_recorded);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6203") && f.contains("recording liability")));
    }

    #[test]
    fn no_record_lien_and_levy_predicate_fails() {
        let mut i = valid_base();
        i.record_format = RecordFormat::NoRecord;
        let r = check(&i);
        assert!(!r.lien_and_levy_predicate_satisfied);
    }

    #[test]
    fn unsigned_record_assessment_invalid() {
        let mut i = valid_base();
        i.officer_signed = false;
        let r = check(&i);
        assert!(!r.assessment_validly_recorded);
        assert!(!r.officer_signature_present);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 301.6203-1") && f.contains("SIGNING")));
    }

    #[test]
    fn missing_taxpayer_identification_invalid() {
        let mut i = valid_base();
        i.taxpayer_identified = false;
        let r = check(&i);
        assert!(!r.assessment_validly_recorded);
        assert!(!r.taxpayer_identification_present);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("IDENTIFICATION OF THE TAXPAYER")));
    }

    #[test]
    fn missing_character_of_liability_invalid() {
        let mut i = valid_base();
        i.character_of_liability_specified = false;
        let r = check(&i);
        assert!(!r.assessment_validly_recorded);
        assert!(!r.character_of_liability_present);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("CHARACTER OF THE LIABILITY")));
    }

    #[test]
    fn missing_taxable_period_when_applicable_invalid() {
        let mut i = valid_base();
        i.taxable_period_identified = false;
        i.taxable_period_applicable = true;
        let r = check(&i);
        assert!(!r.assessment_validly_recorded);
        assert!(!r.taxable_period_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("TAXABLE PERIOD")));
    }

    #[test]
    fn missing_taxable_period_when_not_applicable_valid() {
        let mut i = valid_base();
        i.taxable_period_identified = false;
        i.taxable_period_applicable = false;
        let r = check(&i);
        assert!(r.assessment_validly_recorded);
        assert!(r.taxable_period_compliant);
    }

    #[test]
    fn missing_amount_invalid() {
        let mut i = valid_base();
        i.amount_specified = false;
        let r = check(&i);
        assert!(!r.assessment_validly_recorded);
        assert!(!r.amount_present);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("AMOUNT OF THE ASSESSMENT")));
    }

    #[test]
    fn no_taxpayer_copy_request_no_obligation() {
        let mut i = valid_base();
        i.taxpayer_requested_copy = false;
        i.taxpayer_copy_document = TaxpayerCopyDocument::NoneProvided;
        let r = check(&i);
        assert!(r.taxpayer_copy_request_satisfied);
    }

    #[test]
    fn taxpayer_request_form_4340_satisfies() {
        let mut i = valid_base();
        i.taxpayer_requested_copy = true;
        i.taxpayer_copy_document = TaxpayerCopyDocument::Form4340;
        let r = check(&i);
        assert!(r.taxpayer_copy_request_satisfied);
    }

    #[test]
    fn taxpayer_request_master_file_transcript_satisfies() {
        let mut i = valid_base();
        i.taxpayer_requested_copy = true;
        i.taxpayer_copy_document = TaxpayerCopyDocument::MasterFileTranscript;
        let r = check(&i);
        assert!(r.taxpayer_copy_request_satisfied);
    }

    #[test]
    fn taxpayer_request_no_document_violation() {
        let mut i = valid_base();
        i.taxpayer_requested_copy = true;
        i.taxpayer_copy_document = TaxpayerCopyDocument::NoneProvided;
        let r = check(&i);
        assert!(!r.taxpayer_copy_request_satisfied);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6203") && f.contains("SHALL furnish")));
    }

    #[test]
    fn taxpayer_request_form_23c_copy_invalid_response() {
        let mut i = valid_base();
        i.taxpayer_requested_copy = true;
        i.taxpayer_copy_document = TaxpayerCopyDocument::Form23cCopy;
        let r = check(&i);
        assert!(!r.taxpayer_copy_request_satisfied);
        assert!(r.failure_reasons.iter().any(|f| f.contains("Form 23-C")
            && f.contains("NOT released")
            && f.contains("Rev. Rul. 2007-21")));
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&valid_base());
        assert!(r.citation.contains("§ 6203"));
        assert!(r.citation.contains("26 CFR § 301.6203-1"));
        assert!(r.citation.contains("Rev. Rul. 2007-21"));
        assert!(r.citation.contains("Form 23-C"));
        assert!(r.citation.contains("Form 4340"));
        assert!(r.citation.contains("§ 6201"));
        assert!(r.citation.contains("§ 6303"));
        assert!(r.citation.contains("§ 6321"));
        assert!(r.citation.contains("§ 6331"));
        assert!(r.citation.contains("§ 6502"));
        assert!(r.citation.contains("March v. IRS"));
        assert!(r.citation.contains("335 F.3d 1186"));
    }

    #[test]
    fn note_pins_general_rule() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6203")
            && n.contains("recording liability")
            && n.contains("SHALL furnish")
            || (n.contains("§ 6203")
                && n.contains("recording liability")
                && n.contains("Secretary"))));
    }

    #[test]
    fn note_pins_four_regulatory_elements() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 301.6203-1")
            && n.contains("identification of taxpayer")
            && n.contains("character of liability")
            && n.contains("taxable period")
            && n.contains("amount of assessment")));
    }

    #[test]
    fn note_pins_date_of_assessment_triggers_collection_constellation() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("date of assessment")
            && n.contains("§ 6303(a) 60-day")
            && n.contains("§ 6502")
            && n.contains("10-year")));
    }

    #[test]
    fn note_pins_form_23c_not_released() {
        let r = check(&valid_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Form 23-C") && n.contains("NOT released")));
    }

    #[test]
    fn note_pins_form_4340_presumptive_evidence() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("Form 4340")
            && n.contains("presumptive evidence")
            && n.contains("March v. IRS")));
    }

    #[test]
    fn note_pins_rev_rul_2007_21() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("Rev. Rul. 2007-21")
            && n.contains("not required to provide any particular form")));
    }

    #[test]
    fn note_pins_lien_and_levy_predicate_chain() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6303")
            && n.contains("§ 6321")
            && n.contains("§ 6331")
            && n.contains("§ 6502")));
    }

    #[test]
    fn multiple_element_failures_stack() {
        let mut i = valid_base();
        i.taxpayer_identified = false;
        i.character_of_liability_specified = false;
        i.amount_specified = false;
        let r = check(&i);
        assert!(!r.assessment_validly_recorded);
        assert_eq!(r.failure_reasons.len(), 3);
    }

    #[test]
    fn record_format_truth_table() {
        for (format, exp_record_exists) in [
            (RecordFormat::Form23cSigned, true),
            (RecordFormat::ComputerGeneratedSummaryRecord, true),
            (RecordFormat::NoRecord, false),
        ] {
            let mut i = valid_base();
            i.record_format = format;
            let r = check(&i);
            assert_eq!(r.assessment_validly_recorded, exp_record_exists);
        }
    }

    #[test]
    fn taxpayer_copy_document_truth_table() {
        for (doc, exp_satisfies) in [
            (TaxpayerCopyDocument::Form4340, true),
            (TaxpayerCopyDocument::MasterFileTranscript, true),
            (TaxpayerCopyDocument::Form23cCopy, false),
            (TaxpayerCopyDocument::NoneProvided, false),
        ] {
            let mut i = valid_base();
            i.taxpayer_requested_copy = true;
            i.taxpayer_copy_document = doc;
            let r = check(&i);
            assert_eq!(
                r.taxpayer_copy_request_satisfied, exp_satisfies,
                "doc={:?} expected satisfies={}",
                doc, exp_satisfies
            );
        }
    }

    #[test]
    fn four_regulatory_elements_truth_table_all_combinations() {
        for (tp, ch, period_id, period_app, amt, exp_all_four) in [
            (true, true, true, true, true, true),
            (false, true, true, true, true, false),
            (true, false, true, true, true, false),
            (true, true, false, true, true, false),
            (true, true, false, false, true, true),
            (true, true, true, true, false, false),
        ] {
            let mut i = valid_base();
            i.taxpayer_identified = tp;
            i.character_of_liability_specified = ch;
            i.taxable_period_identified = period_id;
            i.taxable_period_applicable = period_app;
            i.amount_specified = amt;
            let r = check(&i);
            assert_eq!(
                r.all_four_regulatory_elements_present, exp_all_four,
                "tp={} ch={} period_id={} period_app={} amt={}",
                tp, ch, period_id, period_app, amt
            );
        }
    }

    #[test]
    fn unsigned_record_blocks_lien_and_levy_predicate_invariant() {
        let mut i = valid_base();
        i.officer_signed = false;
        let r = check(&i);
        assert!(!r.lien_and_levy_predicate_satisfied);
    }

    #[test]
    fn form_23c_release_request_uniquely_triggers_irs_policy_failure_invariant() {
        let mut i = valid_base();
        i.taxpayer_requested_copy = true;

        i.taxpayer_copy_document = TaxpayerCopyDocument::Form4340;
        let r4340 = check(&i);
        assert!(r4340.taxpayer_copy_request_satisfied);

        i.taxpayer_copy_document = TaxpayerCopyDocument::Form23cCopy;
        let r23c = check(&i);
        assert!(!r23c.taxpayer_copy_request_satisfied);
    }

    #[test]
    fn signed_record_with_all_elements_satisfies_lien_predicate_invariant() {
        let r = check(&valid_base());
        assert!(r.assessment_validly_recorded);
        assert!(r.lien_and_levy_predicate_satisfied);
        assert!(r.officer_signature_present);
        assert!(r.all_four_regulatory_elements_present);
    }
}
