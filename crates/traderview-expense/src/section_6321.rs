//! IRC § 6321 — Lien for taxes (foundational IRS general
//! tax lien). When IRS (1) assesses tax + (2) issues notice
//! and demand for payment + (3) taxpayer neglects or refuses
//! to pay, a federal tax lien arises automatically by
//! operation of law in favor of the United States upon ALL
//! property and rights to property of the taxpayer, whether
//! real or personal, tangible or intangible. Lien relates
//! back to the assessment date.
//!
//! Trader-relevant: trader-landlords face automatic lien
//! exposure when assessment + demand + non-payment converge,
//! and the lien attaches without IRS having to file a Notice
//! of Federal Tax Lien (NFTL). NFTL filing affects PRIORITY
//! against third parties under § 6323 but not lien
//! ATTACHMENT under § 6321.
//!
//! Foundational lien provision tying to the lien
//! constellation:
//! - § 6321 — lien arises (this module)
//! - § 6322 — period of lien (continues until liability
//!   satisfied OR becomes unenforceable by lapse of time)
//! - § 6323 — validity / priority against third parties
//! - § 6325 — release of lien / discharge of property
//! - § 6334 — property exempt from levy (companion module)
//! - § 7426 — third-party wrongful levy (companion module)
//! - § 7433 — civil damages for unauthorized collection
//!
//! **§ 6321 three-element test** (all required):
//! 1. Assessment by the IRS (under § 6201 et seq.)
//! 2. Notice and demand for payment (under § 6303)
//! 3. Taxpayer neglects or refuses to pay after demand
//!
//! When all three are present, lien arises AUTOMATICALLY in
//! favor of the United States upon ALL property and rights
//! to property of the taxpayer, both real AND personal,
//! tangible AND intangible. Lien is statutory; NFTL filing
//! is NOT required for lien to ATTACH (only for priority
//! against third parties under § 6323).
//!
//! **Drye v. United States, 528 U.S. 49 (1999) state-law
//! property-rights doctrine** — federal tax lien attaches to
//! whatever interest in property state law gives the
//! taxpayer; United States v. Craft, 535 U.S. 274 (2002) —
//! tenancy by entirety property still subject to lien for
//! one spouse's tax liability.
//!
//! Citations: 26 USC § 6321 (lien for taxes); § 6322 (period
//! of lien); § 6323 (validity/priority); § 6325 (release);
//! § 6201 (assessment authority); § 6303 (notice and demand);
//! Drye v. United States, 528 U.S. 49 (1999); United States
//! v. Craft, 535 U.S. 274 (2002); IRM 5.17.2 (Federal Tax
//! Liens).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Section6321Input {
    /// Whether IRS has made an assessment under § 6201.
    pub assessment_made: bool,
    /// Whether IRS has issued notice and demand for payment
    /// under § 6303.
    pub notice_and_demand_issued: bool,
    /// Whether taxpayer has neglected or refused to pay
    /// after demand.
    pub neglected_or_refused_to_pay: bool,
    /// Whether IRS has filed Notice of Federal Tax Lien
    /// (NFTL) under § 6323(f) (affects priority, NOT
    /// attachment).
    pub nftl_filed: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6321Result {
    pub lien_arises: bool,
    pub attaches_to_all_property: bool,
    pub relates_back_to_assessment_date: bool,
    pub nftl_required_for_attachment: bool,
    pub nftl_filed: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section6321Input) -> Section6321Result {
    let mut failure_reasons: Vec<String> = Vec::new();

    if !input.assessment_made {
        failure_reasons.push(
            "26 USC § 6321 element 1 — no assessment by IRS under § 6201 et seq.".to_string(),
        );
    }
    if !input.notice_and_demand_issued {
        failure_reasons.push(
            "26 USC § 6321 element 2 — no notice and demand for payment issued under § 6303"
                .to_string(),
        );
    }
    if !input.neglected_or_refused_to_pay {
        failure_reasons.push(
            "26 USC § 6321 element 3 — taxpayer has not neglected or refused to pay after demand"
                .to_string(),
        );
    }

    let lien_arises = failure_reasons.is_empty();

    let notes: Vec<String> = vec![
        "26 USC § 6321 — lien arises automatically when (1) assessment + (2) notice and demand + (3) neglect or refusal to pay; attaches to ALL property and rights to property, whether real or personal, tangible or intangible"
            .to_string(),
        "26 USC § 6321 — lien is statutory and automatic; NFTL filing under § 6323(f) is NOT required for lien to ATTACH (only for priority against third parties under § 6323)"
            .to_string(),
        "26 USC § 6322 — lien continues until liability satisfied OR becomes unenforceable by reason of lapse of time (paired with § 6502 10-year CSED)"
            .to_string(),
        "Drye v. United States, 528 U.S. 49 (1999) — federal tax lien attaches to whatever interest in property state law gives the taxpayer; United States v. Craft, 535 U.S. 274 (2002) — tenancy by entirety property still subject to lien for one spouse's tax liability"
            .to_string(),
    ];

    Section6321Result {
        lien_arises,
        attaches_to_all_property: lien_arises,
        relates_back_to_assessment_date: lien_arises,
        nftl_required_for_attachment: false,
        nftl_filed: input.nftl_filed,
        failure_reasons,
        citation: "26 USC §§ 6321, 6322, 6323, 6325; § 6201; § 6303; Drye v. United States, 528 U.S. 49 (1999); United States v. Craft, 535 U.S. 274 (2002); IRM 5.17.2",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_three_elements() -> Section6321Input {
        Section6321Input {
            assessment_made: true,
            notice_and_demand_issued: true,
            neglected_or_refused_to_pay: true,
            nftl_filed: false,
        }
    }

    #[test]
    fn all_three_elements_lien_arises() {
        let r = check(&all_three_elements());
        assert!(r.lien_arises);
        assert!(r.attaches_to_all_property);
        assert!(r.relates_back_to_assessment_date);
        assert!(!r.nftl_required_for_attachment);
        assert!(r.failure_reasons.is_empty());
    }

    #[test]
    fn no_assessment_no_lien() {
        let mut i = all_three_elements();
        i.assessment_made = false;
        let r = check(&i);
        assert!(!r.lien_arises);
        assert!(!r.attaches_to_all_property);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("element 1") && f.contains("§ 6201")));
    }

    #[test]
    fn no_notice_and_demand_no_lien() {
        let mut i = all_three_elements();
        i.notice_and_demand_issued = false;
        let r = check(&i);
        assert!(!r.lien_arises);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("element 2") && f.contains("§ 6303")));
    }

    #[test]
    fn no_neglect_or_refusal_no_lien() {
        let mut i = all_three_elements();
        i.neglected_or_refused_to_pay = false;
        let r = check(&i);
        assert!(!r.lien_arises);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("element 3") && f.contains("neglected or refused")));
    }

    #[test]
    fn lien_arises_without_nftl_filing() {
        let mut i = all_three_elements();
        i.nftl_filed = false;
        let r = check(&i);
        assert!(r.lien_arises);
        assert!(!r.nftl_filed);
        assert!(!r.nftl_required_for_attachment);
    }

    #[test]
    fn lien_arises_with_nftl_filing_independent() {
        let mut i = all_three_elements();
        i.nftl_filed = true;
        let r = check(&i);
        assert!(r.lien_arises);
        assert!(r.nftl_filed);
        assert!(!r.nftl_required_for_attachment);
    }

    #[test]
    fn nftl_required_for_attachment_always_false() {
        for nftl in [false, true] {
            for elem1 in [false, true] {
                for elem2 in [false, true] {
                    for elem3 in [false, true] {
                        let i = Section6321Input {
                            assessment_made: elem1,
                            notice_and_demand_issued: elem2,
                            neglected_or_refused_to_pay: elem3,
                            nftl_filed: nftl,
                        };
                        let r = check(&i);
                        assert!(!r.nftl_required_for_attachment);
                    }
                }
            }
        }
    }

    #[test]
    fn three_element_truth_table_2x2x2() {
        let mut counter = 0;
        for elem1 in [false, true] {
            for elem2 in [false, true] {
                for elem3 in [false, true] {
                    let i = Section6321Input {
                        assessment_made: elem1,
                        notice_and_demand_issued: elem2,
                        neglected_or_refused_to_pay: elem3,
                        nftl_filed: false,
                    };
                    let r = check(&i);
                    let all_three = elem1 && elem2 && elem3;
                    assert_eq!(r.lien_arises, all_three);
                    if all_three {
                        counter += 1;
                    }
                }
            }
        }
        assert_eq!(counter, 1);
    }

    #[test]
    fn three_failures_stack_when_all_three_missing() {
        let i = Section6321Input {
            assessment_made: false,
            notice_and_demand_issued: false,
            neglected_or_refused_to_pay: false,
            nftl_filed: false,
        };
        let r = check(&i);
        assert!(!r.lien_arises);
        assert_eq!(r.failure_reasons.len(), 3);
    }

    #[test]
    fn citation_pins_lien_constellation() {
        let r = check(&all_three_elements());
        assert!(r.citation.contains("§§ 6321"));
        assert!(r.citation.contains("6322"));
        assert!(r.citation.contains("6323"));
        assert!(r.citation.contains("6325"));
        assert!(r.citation.contains("§ 6201"));
        assert!(r.citation.contains("§ 6303"));
        assert!(r.citation.contains("Drye"));
        assert!(r.citation.contains("Craft"));
        assert!(r.citation.contains("IRM 5.17.2"));
    }

    #[test]
    fn note_pins_three_element_test() {
        let r = check(&all_three_elements());
        assert!(r.notes.iter().any(|n| n.contains("(1) assessment")
            && n.contains("(2) notice and demand")
            && n.contains("(3) neglect or refusal")
            && n.contains("real or personal")
            && n.contains("tangible or intangible")));
    }

    #[test]
    fn note_pins_nftl_priority_vs_attachment_distinction() {
        let r = check(&all_three_elements());
        assert!(r.notes.iter().any(|n| n.contains("§ 6323(f)")
            && n.contains("NOT required for lien to ATTACH")
            && n.contains("priority")));
    }

    #[test]
    fn note_pins_section_6322_period_of_lien() {
        let r = check(&all_three_elements());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6322") && n.contains("lapse of time") && n.contains("§ 6502")));
    }

    #[test]
    fn note_pins_drye_and_craft_state_law_doctrine() {
        let r = check(&all_three_elements());
        assert!(r.notes.iter().any(|n| n.contains("Drye")
            && n.contains("528 U.S. 49 (1999)")
            && n.contains("Craft")
            && n.contains("535 U.S. 274 (2002)")
            && n.contains("tenancy by entirety")));
    }

    #[test]
    fn relates_back_to_assessment_date_when_lien_arises() {
        let r = check(&all_three_elements());
        assert!(r.relates_back_to_assessment_date);
    }

    #[test]
    fn relates_back_false_when_lien_does_not_arise() {
        let mut i = all_three_elements();
        i.assessment_made = false;
        let r = check(&i);
        assert!(!r.relates_back_to_assessment_date);
    }

    #[test]
    fn attaches_to_all_property_true_when_lien_arises() {
        let r = check(&all_three_elements());
        assert!(r.attaches_to_all_property);
    }

    #[test]
    fn attaches_to_all_property_false_when_lien_does_not_arise() {
        let mut i = all_three_elements();
        i.notice_and_demand_issued = false;
        let r = check(&i);
        assert!(!r.attaches_to_all_property);
    }

    #[test]
    fn two_of_three_no_lien_assessment_demand_only() {
        let mut i = all_three_elements();
        i.neglected_or_refused_to_pay = false;
        let r = check(&i);
        assert!(!r.lien_arises);
        assert_eq!(r.failure_reasons.len(), 1);
    }

    #[test]
    fn two_of_three_no_lien_assessment_neglect_only() {
        let mut i = all_three_elements();
        i.notice_and_demand_issued = false;
        let r = check(&i);
        assert!(!r.lien_arises);
        assert_eq!(r.failure_reasons.len(), 1);
    }

    #[test]
    fn two_of_three_no_lien_demand_neglect_only() {
        let mut i = all_three_elements();
        i.assessment_made = false;
        let r = check(&i);
        assert!(!r.lien_arises);
        assert_eq!(r.failure_reasons.len(), 1);
    }

    #[test]
    fn nftl_filing_does_not_affect_lien_attachment() {
        let mut i_with_nftl = all_three_elements();
        i_with_nftl.nftl_filed = true;
        let r_with = check(&i_with_nftl);

        let mut i_without_nftl = all_three_elements();
        i_without_nftl.nftl_filed = false;
        let r_without = check(&i_without_nftl);

        assert_eq!(r_with.lien_arises, r_without.lien_arises);
        assert_eq!(
            r_with.attaches_to_all_property,
            r_without.attaches_to_all_property
        );
        assert_eq!(
            r_with.relates_back_to_assessment_date,
            r_without.relates_back_to_assessment_date
        );
    }

    #[test]
    fn nftl_filed_field_preserved_through_result() {
        let mut i = all_three_elements();
        i.nftl_filed = true;
        let r = check(&i);
        assert!(r.nftl_filed);

        i.nftl_filed = false;
        let r = check(&i);
        assert!(!r.nftl_filed);
    }

    #[test]
    fn lien_attaches_to_intangible_property_per_note() {
        let r = check(&all_three_elements());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("real or personal") && n.contains("tangible or intangible")));
    }

    #[test]
    fn missing_assessment_failure_reason_specific() {
        let mut i = all_three_elements();
        i.assessment_made = false;
        let r = check(&i);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.starts_with("26 USC § 6321 element 1")));
    }

    #[test]
    fn missing_demand_failure_reason_specific() {
        let mut i = all_three_elements();
        i.notice_and_demand_issued = false;
        let r = check(&i);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.starts_with("26 USC § 6321 element 2")));
    }

    #[test]
    fn missing_neglect_failure_reason_specific() {
        let mut i = all_three_elements();
        i.neglected_or_refused_to_pay = false;
        let r = check(&i);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.starts_with("26 USC § 6321 element 3")));
    }
}
