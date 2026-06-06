//! IRC § 7522 — Content of tax due, deficiency, and other
//! notices. Added by Taxpayer Bill of Rights of 1988 (TBOR
//! 1, Pub. L. 100-647) § 6233 to require IRS notices to
//! describe the BASIS for, and identify the AMOUNTS of,
//! every component of liability. Trader-procedural-critical
//! because § 7522 is the taxpayer's statutory entitlement
//! to know what the IRS is asserting and why — covering
//! § 6303 notice and demand, § 6212 SNOD, CP2000 Automated
//! Underreporter notices generated from 1099-B / 1099-K /
//! K-1 matching, and the first 30-day Appeals-eligible
//! proposed-deficiency letter. Companion to § 6201
//! (assessment authority), § 6203 (method of assessment),
//! § 6212 (SNOD), § 6303 (notice and demand), § 6155
//! (payment on notice and demand), § 7491 (burden of proof).
//!
//! **§ 7522(a) General rule** — Any notice to which this
//! section applies shall **describe the basis for, and
//! identify the amounts (if any) of**, any:
//! 1. tax due;
//! 2. interest;
//! 3. additional amounts;
//! 4. additions to the tax; AND
//! 5. assessable penalties
//!
//! included in such notice. **An inadequate description
//! under the preceding sentence shall NOT INVALIDATE such
//! notice** — statutory safe harbor; § 7522(a) is a notice-
//! content DUTY but inadequate notice is curable rather than
//! voidable.
//!
//! **§ 7522(b) Notices to which section applies** — this
//! section applies to:
//! 1. **§ 7522(b)(1)** — Any tax due notice or deficiency
//!    notice described in **§ 6155, § 6212, or § 6303** (CP14
//!    notice and demand, Letter 3171/5071C SNOD, etc.).
//! 2. **§ 7522(b)(2)** — Any notice generated out of any
//!    **information return matching program** (CP2000
//!    Automated Underreporter — matches 1099-B broker
//!    reporting + 1099-K payment-card processor reporting +
//!    K-1 partnership reporting against tax return);
//!    response deadline 30 days from notice date (60 days if
//!    taxpayer outside US).
//! 3. **§ 7522(b)(3)** — **The first letter of proposed
//!    deficiency** which allows the taxpayer an opportunity
//!    for administrative review in the **IRS Independent
//!    Office of Appeals** (Letter 525 "30-day letter";
//!    Taxpayer First Act of 2019 § 1001 redesignated as
//!    "Independent Office of Appeals").
//!
//! Citations: 26 USC § 7522; Taxpayer Bill of Rights of
//! 1988 (TBOR 1, Pub. L. 100-647 § 6233); Taxpayer First
//! Act of 2019 § 1001 (Independent Office of Appeals); 26
//! CFR § 301.7522-1; CP2000 IRM 4.19.3; Letter 525 IRM
//! 4.10.7; § 6155; § 6212; § 6303; § 7491; IRS Publication
//! 1 (Your Rights as a Taxpayer).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NoticeType {
    /// § 7522(b)(1) — § 6303 notice and demand (CP14, Letter
    /// 1058).
    Section6303NoticeAndDemand,
    /// § 7522(b)(1) — § 6212 SNOD (Letter 3171, 5071C, etc.).
    Section6212Snod,
    /// § 7522(b)(1) — § 6155 payment on notice and demand.
    Section6155PaymentNotice,
    /// § 7522(b)(2) — CP2000 Automated Underreporter notice
    /// generated from information return matching.
    Cp2000InformationMatching,
    /// § 7522(b)(3) — Letter 525 30-day letter with Appeals
    /// opportunity.
    Letter525ThirtyDayLetter,
    /// Notice not covered by § 7522.
    NotCovered,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section7522Input {
    pub notice_type: NoticeType,
    /// Whether notice describes the BASIS for the tax due
    /// (§ 7522(a) duty #1).
    pub basis_described: bool,
    /// Whether notice identifies AMOUNT of tax due.
    pub tax_amount_identified: bool,
    /// Whether notice identifies AMOUNT of interest.
    pub interest_amount_identified: bool,
    /// Whether notice identifies AMOUNT of additional
    /// amounts.
    pub additional_amounts_identified: bool,
    /// Whether notice identifies AMOUNT of additions to tax.
    pub additions_to_tax_identified: bool,
    /// Whether notice identifies AMOUNT of assessable
    /// penalties.
    pub assessable_penalties_identified: bool,
    /// Whether ANY of the liability components is included
    /// in the notice (if all zero, no amounts required).
    pub any_liability_component_included: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section7522Result {
    pub section_7522_applies: bool,
    pub basis_disclosure_compliant: bool,
    pub all_amounts_identified: bool,
    pub fully_compliant: bool,
    pub safe_harbor_engaged: bool,
    pub notice_remains_valid_despite_inadequacy: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section7522Input) -> Section7522Result {
    let mut failure_reasons: Vec<String> = Vec::new();

    let applies = !matches!(input.notice_type, NoticeType::NotCovered);

    if !applies {
        return Section7522Result {
            section_7522_applies: false,
            basis_disclosure_compliant: true,
            all_amounts_identified: true,
            fully_compliant: true,
            safe_harbor_engaged: false,
            notice_remains_valid_despite_inadequacy: true,
            failure_reasons,
            citation: "26 USC § 7522(b) — section does not apply to this notice type",
            notes: vec![
                "26 USC § 7522(b) — section applies only to (1) § 6155/6212/6303 notices, (2) information return matching program notices (CP2000), and (3) first letter of proposed deficiency with Appeals opportunity (Letter 525)".to_string(),
            ],
        };
    }

    if !input.basis_described {
        failure_reasons.push(
            "26 USC § 7522(a) — notice must DESCRIBE THE BASIS for any tax due, interest, additional amounts, additions to tax, and assessable penalties included in notice".to_string(),
        );
    }

    if input.any_liability_component_included && !input.tax_amount_identified {
        failure_reasons.push(
            "26 USC § 7522(a) — notice must IDENTIFY THE AMOUNTS (if any) of tax due".to_string(),
        );
    }

    if input.any_liability_component_included && !input.interest_amount_identified {
        failure_reasons.push(
            "26 USC § 7522(a) — notice must IDENTIFY THE AMOUNTS (if any) of interest".to_string(),
        );
    }

    if input.any_liability_component_included && !input.additional_amounts_identified {
        failure_reasons.push(
            "26 USC § 7522(a) — notice must IDENTIFY THE AMOUNTS (if any) of additional amounts"
                .to_string(),
        );
    }

    if input.any_liability_component_included && !input.additions_to_tax_identified {
        failure_reasons.push(
            "26 USC § 7522(a) — notice must IDENTIFY THE AMOUNTS (if any) of additions to the tax"
                .to_string(),
        );
    }

    if input.any_liability_component_included && !input.assessable_penalties_identified {
        failure_reasons.push(
            "26 USC § 7522(a) — notice must IDENTIFY THE AMOUNTS (if any) of assessable penalties"
                .to_string(),
        );
    }

    let all_amounts = input.tax_amount_identified
        && input.interest_amount_identified
        && input.additional_amounts_identified
        && input.additions_to_tax_identified
        && input.assessable_penalties_identified;

    let fully_compliant = input.basis_described && all_amounts;
    let inadequate = !fully_compliant;
    let safe_harbor = inadequate;

    let notes: Vec<String> = vec![
        "26 USC § 7522(a) — any covered notice shall describe the basis for, and identify the amounts (if any) of, any tax due, interest, additional amounts, additions to the tax, and assessable penalties".to_string(),
        "26 USC § 7522(a) safe harbor — 'an inadequate description under the preceding sentence shall NOT INVALIDATE such notice'; § 7522(a) is a notice-content DUTY but inadequate notice is curable rather than voidable".to_string(),
        "26 USC § 7522(b)(1) — applies to § 6155 payment on notice and demand, § 6212 SNOD (90-day letter), § 6303 notice and demand (CP14, Letter 1058)".to_string(),
        "26 USC § 7522(b)(2) — applies to information return matching program notices: CP2000 Automated Underreporter generated from 1099-B + 1099-K + K-1 third-party reporting matched against taxpayer return; response deadline 30 days from notice date (60 days outside US)".to_string(),
        "26 USC § 7522(b)(3) — applies to first letter of proposed deficiency with administrative review opportunity in IRS Independent Office of Appeals (Letter 525 30-day letter); Taxpayer First Act of 2019 § 1001 redesignated 'Office of Appeals' as 'Independent Office of Appeals'".to_string(),
        "Taxpayer Bill of Rights of 1988 (TBOR 1, Pub. L. 100-647 § 6233) added § 7522; predecessor of IRS Publication 1 'Your Rights as a Taxpayer'".to_string(),
        "Cross-references: § 7522 is content-disclosure layer over § 6201 (assessment authority) + § 6203 (method of assessment) + § 6212 (SNOD) + § 6303 (notice and demand); inadequate § 7522 description does not extend § 6213(a) Tax Court petition window".to_string(),
    ];

    Section7522Result {
        section_7522_applies: applies,
        basis_disclosure_compliant: input.basis_described,
        all_amounts_identified: all_amounts || !input.any_liability_component_included,
        fully_compliant,
        safe_harbor_engaged: safe_harbor,
        notice_remains_valid_despite_inadequacy: safe_harbor,
        failure_reasons,
        citation: "26 USC § 7522; TBOR 1 (Pub. L. 100-647 § 6233); Taxpayer First Act of 2019 § 1001; 26 CFR § 301.7522-1; IRM 4.19.3 (CP2000); IRM 4.10.7 (Letter 525); § 6155; § 6212; § 6303; § 7491; IRS Publication 1",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fully_compliant_base() -> Section7522Input {
        Section7522Input {
            notice_type: NoticeType::Section6212Snod,
            basis_described: true,
            tax_amount_identified: true,
            interest_amount_identified: true,
            additional_amounts_identified: true,
            additions_to_tax_identified: true,
            assessable_penalties_identified: true,
            any_liability_component_included: true,
        }
    }

    #[test]
    fn fully_compliant_snod_passes() {
        let r = check(&fully_compliant_base());
        assert!(r.section_7522_applies);
        assert!(r.fully_compliant);
        assert!(!r.safe_harbor_engaged);
    }

    #[test]
    fn section_6303_notice_covered() {
        let mut i = fully_compliant_base();
        i.notice_type = NoticeType::Section6303NoticeAndDemand;
        let r = check(&i);
        assert!(r.section_7522_applies);
        assert!(r.fully_compliant);
    }

    #[test]
    fn section_6155_payment_notice_covered() {
        let mut i = fully_compliant_base();
        i.notice_type = NoticeType::Section6155PaymentNotice;
        let r = check(&i);
        assert!(r.section_7522_applies);
    }

    #[test]
    fn cp2000_information_matching_covered() {
        let mut i = fully_compliant_base();
        i.notice_type = NoticeType::Cp2000InformationMatching;
        let r = check(&i);
        assert!(r.section_7522_applies);
    }

    #[test]
    fn letter_525_thirty_day_letter_covered() {
        let mut i = fully_compliant_base();
        i.notice_type = NoticeType::Letter525ThirtyDayLetter;
        let r = check(&i);
        assert!(r.section_7522_applies);
    }

    #[test]
    fn not_covered_notice_section_not_engaged() {
        let mut i = fully_compliant_base();
        i.notice_type = NoticeType::NotCovered;
        let r = check(&i);
        assert!(!r.section_7522_applies);
        assert!(r.fully_compliant);
    }

    #[test]
    fn missing_basis_description_violation_with_safe_harbor() {
        let mut i = fully_compliant_base();
        i.basis_described = false;
        let r = check(&i);
        assert!(!r.fully_compliant);
        assert!(r.safe_harbor_engaged);
        assert!(r.notice_remains_valid_despite_inadequacy);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 7522(a)") && f.contains("DESCRIBE THE BASIS")));
    }

    #[test]
    fn missing_tax_amount_violation() {
        let mut i = fully_compliant_base();
        i.tax_amount_identified = false;
        let r = check(&i);
        assert!(!r.fully_compliant);
        assert!(r.failure_reasons.iter().any(|f| f.contains("tax due")));
    }

    #[test]
    fn missing_interest_amount_violation() {
        let mut i = fully_compliant_base();
        i.interest_amount_identified = false;
        let r = check(&i);
        assert!(!r.fully_compliant);
        assert!(r.failure_reasons.iter().any(|f| f.contains("interest")));
    }

    #[test]
    fn missing_additional_amounts_violation() {
        let mut i = fully_compliant_base();
        i.additional_amounts_identified = false;
        let r = check(&i);
        assert!(!r.fully_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("additional amounts")));
    }

    #[test]
    fn missing_additions_to_tax_violation() {
        let mut i = fully_compliant_base();
        i.additions_to_tax_identified = false;
        let r = check(&i);
        assert!(!r.fully_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("additions to the tax")));
    }

    #[test]
    fn missing_assessable_penalties_violation() {
        let mut i = fully_compliant_base();
        i.assessable_penalties_identified = false;
        let r = check(&i);
        assert!(!r.fully_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("assessable penalties")));
    }

    #[test]
    fn safe_harbor_preserves_notice_validity() {
        let mut i = fully_compliant_base();
        i.basis_described = false;
        i.tax_amount_identified = false;
        i.interest_amount_identified = false;
        let r = check(&i);
        assert!(!r.fully_compliant);
        assert!(r.safe_harbor_engaged);
        assert!(r.notice_remains_valid_despite_inadequacy);
    }

    #[test]
    fn no_liability_component_no_amount_required() {
        let mut i = fully_compliant_base();
        i.any_liability_component_included = false;
        i.tax_amount_identified = false;
        i.interest_amount_identified = false;
        i.additional_amounts_identified = false;
        i.additions_to_tax_identified = false;
        i.assessable_penalties_identified = false;
        let r = check(&i);
        assert!(r.all_amounts_identified);
        assert_eq!(r.failure_reasons.len(), 0);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&fully_compliant_base());
        assert!(r.citation.contains("§ 7522"));
        assert!(r.citation.contains("TBOR 1"));
        assert!(r.citation.contains("Pub. L. 100-647 § 6233"));
        assert!(r.citation.contains("Taxpayer First Act of 2019 § 1001"));
        assert!(r.citation.contains("§ 301.7522-1"));
        assert!(r.citation.contains("IRM 4.19.3"));
        assert!(r.citation.contains("IRM 4.10.7"));
        assert!(r.citation.contains("§ 6155"));
        assert!(r.citation.contains("§ 6212"));
        assert!(r.citation.contains("§ 6303"));
        assert!(r.citation.contains("§ 7491"));
        assert!(r.citation.contains("IRS Publication 1"));
    }

    #[test]
    fn note_pins_general_rule_describe_basis_and_amounts() {
        let r = check(&fully_compliant_base());
        assert!(r.notes.iter().any(|n| n.contains("describe the basis")
            && n.contains("identify the amounts")
            && n.contains("tax due")
            && n.contains("interest")
            && n.contains("additional amounts")
            && n.contains("additions to the tax")
            && n.contains("assessable penalties")));
    }

    #[test]
    fn note_pins_safe_harbor() {
        let r = check(&fully_compliant_base());
        assert!(r.notes.iter().any(|n| n.contains("NOT INVALIDATE")
            && n.contains("safe harbor")
            && n.contains("curable")));
    }

    #[test]
    fn note_pins_section_b1_covered_notices() {
        let r = check(&fully_compliant_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 7522(b)(1)")
            && n.contains("§ 6155")
            && n.contains("§ 6212")
            && n.contains("§ 6303")
            && n.contains("CP14")
            && n.contains("Letter 1058")));
    }

    #[test]
    fn note_pins_section_b2_cp2000_with_third_party_info_returns() {
        let r = check(&fully_compliant_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 7522(b)(2)")
            && n.contains("CP2000")
            && n.contains("1099-B")
            && n.contains("1099-K")
            && n.contains("K-1")
            && n.contains("30 days")
            && n.contains("60 days outside US")));
    }

    #[test]
    fn note_pins_section_b3_letter_525_with_independent_office_of_appeals() {
        let r = check(&fully_compliant_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 7522(b)(3)")
            && n.contains("Letter 525")
            && n.contains("Independent Office of Appeals")
            && n.contains("Taxpayer First Act of 2019 § 1001")));
    }

    #[test]
    fn note_pins_tbor_1_origin() {
        let r = check(&fully_compliant_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Taxpayer Bill of Rights of 1988")
                && n.contains("TBOR 1")
                && n.contains("§ 6233")
                && n.contains("IRS Publication 1")));
    }

    #[test]
    fn note_pins_content_disclosure_layer_over_collection_constellation() {
        let r = check(&fully_compliant_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6201")
            && n.contains("§ 6203")
            && n.contains("§ 6212")
            && n.contains("§ 6303")
            && n.contains("§ 6213(a)")));
    }

    #[test]
    fn multiple_amount_failures_stack() {
        let mut i = fully_compliant_base();
        i.tax_amount_identified = false;
        i.interest_amount_identified = false;
        i.additions_to_tax_identified = false;
        let r = check(&i);
        assert!(!r.fully_compliant);
        assert_eq!(r.failure_reasons.len(), 3);
    }

    #[test]
    fn notice_type_truth_table_six_cells() {
        for (notice_type, exp_applies) in [
            (NoticeType::Section6303NoticeAndDemand, true),
            (NoticeType::Section6212Snod, true),
            (NoticeType::Section6155PaymentNotice, true),
            (NoticeType::Cp2000InformationMatching, true),
            (NoticeType::Letter525ThirtyDayLetter, true),
            (NoticeType::NotCovered, false),
        ] {
            let mut i = fully_compliant_base();
            i.notice_type = notice_type;
            let r = check(&i);
            assert_eq!(
                r.section_7522_applies, exp_applies,
                "notice_type={:?} expected applies={}",
                notice_type, exp_applies
            );
        }
    }

    #[test]
    fn all_five_amounts_truth_table_when_liability_included() {
        for (tax, interest, additional, additions, penalties, exp_all_amounts) in [
            (true, true, true, true, true, true),
            (false, true, true, true, true, false),
            (true, false, true, true, true, false),
            (true, true, false, true, true, false),
            (true, true, true, false, true, false),
            (true, true, true, true, false, false),
            (false, false, false, false, false, false),
        ] {
            let mut i = fully_compliant_base();
            i.tax_amount_identified = tax;
            i.interest_amount_identified = interest;
            i.additional_amounts_identified = additional;
            i.additions_to_tax_identified = additions;
            i.assessable_penalties_identified = penalties;
            let r = check(&i);
            assert_eq!(
                r.all_amounts_identified, exp_all_amounts,
                "tax={} interest={} additional={} additions={} penalties={}",
                tax, interest, additional, additions, penalties
            );
        }
    }

    #[test]
    fn inadequate_description_always_engages_safe_harbor_invariant() {
        let mut i = fully_compliant_base();
        i.basis_described = false;
        let r = check(&i);
        assert!(r.safe_harbor_engaged);
        assert!(r.notice_remains_valid_despite_inadequacy);
    }

    #[test]
    fn fully_compliant_notice_does_not_engage_safe_harbor_invariant() {
        let r = check(&fully_compliant_base());
        assert!(r.fully_compliant);
        assert!(!r.safe_harbor_engaged);
    }

    #[test]
    fn not_covered_notice_returns_compliant_without_engaging_section_invariant() {
        let mut i = fully_compliant_base();
        i.notice_type = NoticeType::NotCovered;
        i.basis_described = false;
        i.tax_amount_identified = false;
        let r = check(&i);
        assert!(!r.section_7522_applies);
        assert!(r.fully_compliant);
        assert_eq!(r.failure_reasons.len(), 0);
    }
}
