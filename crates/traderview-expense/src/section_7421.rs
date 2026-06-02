//! IRC § 7421 — Anti-Injunction Act (AIA). General bar:
//! "no suit for the purpose of restraining the assessment or
//! collection of any tax shall be maintained in any court by
//! any person." Trader-procedural-critical: when can a
//! trader-taxpayer obtain a temporary restraining order
//! (TRO) or preliminary injunction against an IRS levy,
//! lien, or assessment? Default answer = NEVER, with narrow
//! statutory and judicial exceptions. Procedural-companion
//! to § 7521 (interview procedure), § 7525 (FATP privilege),
//! § 7811 (Taxpayer Assistance Orders), and § 7430 (litigation
//! costs).
//!
//! **§ 7421(a) general bar** — except as provided in the
//! enumerated statutory exceptions, no suit for the purpose
//! of restraining the assessment or collection of any tax
//! shall be maintained in any court by any person, whether or
//! not such person is the person against whom such tax was
//! assessed.
//!
//! **Eleven statutory exceptions** (verbatim from § 7421(a)):
//! 1. § 6015(e) — innocent spouse relief review
//! 2. § 6212(a) + (c) — SNOD Tax Court review
//! 3. § 6213(a) — pre-assessment Tax Court restraint
//! 4. § 6232(c) — partnership-level adjustment review
//! 5. § 6330(e)(1) — Collection Due Process action suspension
//! 6. § 6331(i) — levy on bond proceeds (uniformed services)
//! 7. § 6672(c) — TFRP collection due process review
//! 8. § 6694(c) — preparer penalty collection due process
//! 9. § 7426(a) + (b)(1) — third-party wrongful levy
//! 10. § 7429(b) — jeopardy assessment review
//! 11. § 7436 — employment-status determination review
//!
//! **Enochs v. Williams Packing, 370 U.S. 1 (1962) judicial
//! two-prong exception**:
//! Prong 1 — under no circumstances could the Government
//! ultimately prevail.
//! Prong 2 — equity jurisdiction otherwise exists (irreparable
//! harm + legal remedy inadequate).
//! BOTH prongs required conjunctively.
//!
//! **CIC Services v. IRS, 593 U.S. 209 (2021) regulatory
//! pre-enforcement carve-out**: a suit challenging the
//! lawfulness of an IRS reporting requirement is NOT a suit
//! to restrain assessment or collection within the meaning of
//! § 7421(a). The Court drew distinction between (1) suits
//! attacking the assessment/collection itself versus (2) suits
//! challenging regulatory information-reporting obligations.
//!
//! Citations: 26 USC § 7421(a); Enochs v. Williams Packing &
//! Nav. Co., 370 U.S. 1 (1962); CIC Services, LLC v. IRS, 593
//! U.S. 209 (2021); South Carolina v. Regan, 465 U.S. 367
//! (1984) (third-party-no-alternative narrow exception).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StatutoryException {
    Section6015e,
    Section6212,
    Section6213a,
    Section6232c,
    Section6330e1,
    Section6331i,
    Section6672c,
    Section6694c,
    Section7426,
    Section7429b,
    Section7436,
    None,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SuitPurpose {
    /// Restrain assessment of tax (§ 7421(a) bar engaged).
    RestrainAssessment,
    /// Restrain collection of tax (§ 7421(a) bar engaged).
    RestrainCollection,
    /// Pre-enforcement challenge to IRS reporting
    /// requirement / regulation (CIC Services carve-out).
    RegulatoryPreEnforcement,
    /// Refund suit after payment (NOT barred by § 7421(a)).
    RefundAfterPayment,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section7421Input {
    pub suit_purpose: SuitPurpose,
    pub statutory_exception: StatutoryException,
    /// Enochs prong 1 — under no circumstances could the
    /// Government ultimately prevail.
    pub government_cannot_prevail: bool,
    /// Enochs prong 2 — equity jurisdiction otherwise
    /// exists (irreparable harm + legal remedy inadequate).
    pub equity_jurisdiction_exists: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section7421Result {
    pub suit_permitted: bool,
    pub aia_bar_engaged: bool,
    pub statutory_exception_engaged: bool,
    pub enochs_exception_engaged: bool,
    pub cic_services_carveout_engaged: bool,
    pub refund_suit_path: bool,
    pub bar_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section7421Input) -> Section7421Result {
    let mut bar_reasons: Vec<String> = Vec::new();

    let assessment_or_collection = matches!(
        input.suit_purpose,
        SuitPurpose::RestrainAssessment | SuitPurpose::RestrainCollection
    );

    let cic_services_carveout = matches!(
        input.suit_purpose,
        SuitPurpose::RegulatoryPreEnforcement
    );

    let refund_suit_path = matches!(input.suit_purpose, SuitPurpose::RefundAfterPayment);

    let statutory_engaged = !matches!(input.statutory_exception, StatutoryException::None);

    let enochs_engaged =
        input.government_cannot_prevail && input.equity_jurisdiction_exists;

    if assessment_or_collection
        && !statutory_engaged
        && !enochs_engaged
        && !cic_services_carveout
    {
        bar_reasons.push(
            "26 USC § 7421(a) — Anti-Injunction Act bars suits to restrain assessment or collection of any tax".to_string(),
        );
    }

    if assessment_or_collection
        && !statutory_engaged
        && !input.government_cannot_prevail
    {
        bar_reasons.push(
            "Enochs v. Williams Packing, 370 U.S. 1 (1962) prong 1 NOT satisfied — government could ultimately prevail under some circumstances".to_string(),
        );
    }

    if assessment_or_collection
        && !statutory_engaged
        && !input.equity_jurisdiction_exists
    {
        bar_reasons.push(
            "Enochs v. Williams Packing, 370 U.S. 1 (1962) prong 2 NOT satisfied — no equity jurisdiction (no irreparable harm OR legal remedy adequate)".to_string(),
        );
    }

    let suit_permitted = !assessment_or_collection
        || statutory_engaged
        || enochs_engaged
        || cic_services_carveout;

    let aia_bar_engaged = assessment_or_collection;

    let notes: Vec<String> = vec![
        "26 USC § 7421(a) — general bar; 11 statutory exceptions: §§ 6015(e), 6212(a)+(c), 6213(a), 6232(c), 6330(e)(1), 6331(i), 6672(c), 6694(c), 7426(a)+(b)(1), 7429(b), 7436"
            .to_string(),
        "Enochs v. Williams Packing, 370 U.S. 1 (1962) — judicial 2-prong exception: (1) government cannot ultimately prevail AND (2) equity jurisdiction exists; BOTH required conjunctively"
            .to_string(),
        "CIC Services v. IRS, 593 U.S. 209 (2021) — pre-enforcement challenge to IRS reporting requirement is NOT a suit to restrain assessment or collection within § 7421(a)"
            .to_string(),
        "default trader procedural pathway — pay disputed tax + file refund claim under § 6402/§ 7422 + file refund suit (NOT barred by § 7421(a)); paired with § 7430 (litigation costs)"
            .to_string(),
    ];

    Section7421Result {
        suit_permitted,
        aia_bar_engaged,
        statutory_exception_engaged: statutory_engaged,
        enochs_exception_engaged: enochs_engaged && assessment_or_collection,
        cic_services_carveout_engaged: cic_services_carveout,
        refund_suit_path,
        bar_reasons,
        citation: "26 USC § 7421(a); Enochs v. Williams Packing, 370 U.S. 1 (1962); CIC Services v. IRS, 593 U.S. 209 (2021)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn restrain_assessment_base() -> Section7421Input {
        Section7421Input {
            suit_purpose: SuitPurpose::RestrainAssessment,
            statutory_exception: StatutoryException::None,
            government_cannot_prevail: false,
            equity_jurisdiction_exists: false,
        }
    }

    #[test]
    fn restrain_assessment_no_exception_barred() {
        let r = check(&restrain_assessment_base());
        assert!(!r.suit_permitted);
        assert!(r.aia_bar_engaged);
        assert!(!r.statutory_exception_engaged);
        assert!(!r.enochs_exception_engaged);
        assert!(!r.cic_services_carveout_engaged);
        assert!(r
            .bar_reasons
            .iter()
            .any(|b| b.contains("§ 7421(a)") && b.contains("Anti-Injunction Act")));
    }

    #[test]
    fn restrain_collection_no_exception_barred() {
        let mut i = restrain_assessment_base();
        i.suit_purpose = SuitPurpose::RestrainCollection;
        let r = check(&i);
        assert!(!r.suit_permitted);
        assert!(r.aia_bar_engaged);
    }

    #[test]
    fn section_6213a_exception_permits_suit() {
        let mut i = restrain_assessment_base();
        i.statutory_exception = StatutoryException::Section6213a;
        let r = check(&i);
        assert!(r.suit_permitted);
        assert!(r.statutory_exception_engaged);
    }

    #[test]
    fn section_6330e1_cdp_exception_permits_suit() {
        let mut i = restrain_assessment_base();
        i.suit_purpose = SuitPurpose::RestrainCollection;
        i.statutory_exception = StatutoryException::Section6330e1;
        let r = check(&i);
        assert!(r.suit_permitted);
        assert!(r.statutory_exception_engaged);
    }

    #[test]
    fn section_7426_wrongful_levy_exception_permits_suit() {
        let mut i = restrain_assessment_base();
        i.suit_purpose = SuitPurpose::RestrainCollection;
        i.statutory_exception = StatutoryException::Section7426;
        let r = check(&i);
        assert!(r.suit_permitted);
        assert!(r.statutory_exception_engaged);
    }

    #[test]
    fn section_7429b_jeopardy_assessment_permits_suit() {
        let mut i = restrain_assessment_base();
        i.statutory_exception = StatutoryException::Section7429b;
        let r = check(&i);
        assert!(r.suit_permitted);
        assert!(r.statutory_exception_engaged);
    }

    #[test]
    fn section_6015e_innocent_spouse_permits_suit() {
        let mut i = restrain_assessment_base();
        i.statutory_exception = StatutoryException::Section6015e;
        let r = check(&i);
        assert!(r.suit_permitted);
        assert!(r.statutory_exception_engaged);
    }

    #[test]
    fn enochs_both_prongs_permits_suit() {
        let mut i = restrain_assessment_base();
        i.government_cannot_prevail = true;
        i.equity_jurisdiction_exists = true;
        let r = check(&i);
        assert!(r.suit_permitted);
        assert!(r.enochs_exception_engaged);
    }

    #[test]
    fn enochs_only_prong_1_barred() {
        let mut i = restrain_assessment_base();
        i.government_cannot_prevail = true;
        i.equity_jurisdiction_exists = false;
        let r = check(&i);
        assert!(!r.suit_permitted);
        assert!(!r.enochs_exception_engaged);
        assert!(r
            .bar_reasons
            .iter()
            .any(|b| b.contains("prong 2 NOT satisfied")));
    }

    #[test]
    fn enochs_only_prong_2_barred() {
        let mut i = restrain_assessment_base();
        i.government_cannot_prevail = false;
        i.equity_jurisdiction_exists = true;
        let r = check(&i);
        assert!(!r.suit_permitted);
        assert!(!r.enochs_exception_engaged);
        assert!(r
            .bar_reasons
            .iter()
            .any(|b| b.contains("prong 1 NOT satisfied")));
    }

    #[test]
    fn cic_services_pre_enforcement_permits_suit() {
        let mut i = restrain_assessment_base();
        i.suit_purpose = SuitPurpose::RegulatoryPreEnforcement;
        let r = check(&i);
        assert!(r.suit_permitted);
        assert!(r.cic_services_carveout_engaged);
        assert!(!r.aia_bar_engaged);
    }

    #[test]
    fn refund_after_payment_not_barred() {
        let mut i = restrain_assessment_base();
        i.suit_purpose = SuitPurpose::RefundAfterPayment;
        let r = check(&i);
        assert!(r.suit_permitted);
        assert!(r.refund_suit_path);
        assert!(!r.aia_bar_engaged);
    }

    #[test]
    fn citation_pins_enochs_and_cic_services() {
        let r = check(&restrain_assessment_base());
        assert!(r.citation.contains("§ 7421(a)"));
        assert!(r.citation.contains("Enochs v. Williams Packing"));
        assert!(r.citation.contains("370 U.S. 1 (1962)"));
        assert!(r.citation.contains("CIC Services"));
        assert!(r.citation.contains("593 U.S. 209 (2021)"));
    }

    #[test]
    fn note_pins_11_statutory_exceptions() {
        let r = check(&restrain_assessment_base());
        assert!(r.notes.iter().any(|n| n.contains("§§ 6015(e)")
            && n.contains("6212(a)+(c)")
            && n.contains("6213(a)")
            && n.contains("6232(c)")
            && n.contains("6330(e)(1)")
            && n.contains("6331(i)")
            && n.contains("6672(c)")
            && n.contains("6694(c)")
            && n.contains("7426(a)+(b)(1)")
            && n.contains("7429(b)")
            && n.contains("7436")));
    }

    #[test]
    fn note_pins_enochs_two_prong_conjunction() {
        let r = check(&restrain_assessment_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Enochs") && n.contains("conjunctively")));
    }

    #[test]
    fn note_pins_cic_services_distinction() {
        let r = check(&restrain_assessment_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("CIC Services") && n.contains("reporting requirement")));
    }

    #[test]
    fn note_pins_refund_suit_pathway() {
        let r = check(&restrain_assessment_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("refund") && n.contains("§ 7422")));
    }

    #[test]
    fn statutory_exception_truth_table() {
        for exc in [
            StatutoryException::Section6015e,
            StatutoryException::Section6212,
            StatutoryException::Section6213a,
            StatutoryException::Section6232c,
            StatutoryException::Section6330e1,
            StatutoryException::Section6331i,
            StatutoryException::Section6672c,
            StatutoryException::Section6694c,
            StatutoryException::Section7426,
            StatutoryException::Section7429b,
            StatutoryException::Section7436,
        ] {
            let mut i = restrain_assessment_base();
            i.statutory_exception = exc;
            let r = check(&i);
            assert!(r.suit_permitted);
            assert!(r.statutory_exception_engaged);
        }
    }

    #[test]
    fn suit_purpose_truth_table() {
        for (purpose, exp_bar, exp_permitted) in [
            (SuitPurpose::RestrainAssessment, true, false),
            (SuitPurpose::RestrainCollection, true, false),
            (SuitPurpose::RegulatoryPreEnforcement, false, true),
            (SuitPurpose::RefundAfterPayment, false, true),
        ] {
            let mut i = restrain_assessment_base();
            i.suit_purpose = purpose;
            let r = check(&i);
            assert_eq!(r.aia_bar_engaged, exp_bar);
            assert_eq!(r.suit_permitted, exp_permitted);
        }
    }

    #[test]
    fn enochs_2x2_truth_table() {
        for (prong1, prong2, exp_permitted) in [
            (true, true, true),
            (true, false, false),
            (false, true, false),
            (false, false, false),
        ] {
            let mut i = restrain_assessment_base();
            i.government_cannot_prevail = prong1;
            i.equity_jurisdiction_exists = prong2;
            let r = check(&i);
            assert_eq!(r.suit_permitted, exp_permitted);
            assert_eq!(r.enochs_exception_engaged, exp_permitted);
        }
    }

    #[test]
    fn enochs_only_engages_for_assessment_or_collection() {
        let mut i = restrain_assessment_base();
        i.suit_purpose = SuitPurpose::RegulatoryPreEnforcement;
        i.government_cannot_prevail = true;
        i.equity_jurisdiction_exists = true;
        let r = check(&i);
        assert!(!r.enochs_exception_engaged);
        assert!(r.cic_services_carveout_engaged);
    }

    #[test]
    fn cic_services_uniquely_permits_pre_enforcement_invariant() {
        let mut i_regulatory = restrain_assessment_base();
        i_regulatory.suit_purpose = SuitPurpose::RegulatoryPreEnforcement;
        let r_regulatory = check(&i_regulatory);
        assert!(r_regulatory.suit_permitted);
        assert!(r_regulatory.cic_services_carveout_engaged);

        let r_assessment = check(&restrain_assessment_base());
        assert!(!r_assessment.suit_permitted);
        assert!(!r_assessment.cic_services_carveout_engaged);
    }

    #[test]
    fn refund_path_uniquely_distinct_from_aia_invariant() {
        let mut i_refund = restrain_assessment_base();
        i_refund.suit_purpose = SuitPurpose::RefundAfterPayment;
        let r_refund = check(&i_refund);
        assert!(r_refund.refund_suit_path);
        assert!(!r_refund.aia_bar_engaged);

        for other in [
            SuitPurpose::RestrainAssessment,
            SuitPurpose::RestrainCollection,
            SuitPurpose::RegulatoryPreEnforcement,
        ] {
            let mut i = restrain_assessment_base();
            i.suit_purpose = other;
            let r = check(&i);
            assert!(!r.refund_suit_path);
        }
    }

    #[test]
    fn three_exception_pathways_independent() {
        let r_statutory = {
            let mut i = restrain_assessment_base();
            i.statutory_exception = StatutoryException::Section6213a;
            check(&i)
        };
        assert!(r_statutory.suit_permitted);
        assert!(r_statutory.statutory_exception_engaged);
        assert!(!r_statutory.enochs_exception_engaged);
        assert!(!r_statutory.cic_services_carveout_engaged);

        let r_enochs = {
            let mut i = restrain_assessment_base();
            i.government_cannot_prevail = true;
            i.equity_jurisdiction_exists = true;
            check(&i)
        };
        assert!(r_enochs.suit_permitted);
        assert!(!r_enochs.statutory_exception_engaged);
        assert!(r_enochs.enochs_exception_engaged);
        assert!(!r_enochs.cic_services_carveout_engaged);

        let r_cic = {
            let mut i = restrain_assessment_base();
            i.suit_purpose = SuitPurpose::RegulatoryPreEnforcement;
            check(&i)
        };
        assert!(r_cic.suit_permitted);
        assert!(!r_cic.statutory_exception_engaged);
        assert!(!r_cic.enochs_exception_engaged);
        assert!(r_cic.cic_services_carveout_engaged);
    }

    #[test]
    fn multiple_failure_reasons_stack_for_bare_assessment_suit() {
        let r = check(&restrain_assessment_base());
        assert_eq!(r.bar_reasons.len(), 3);
        assert!(r.bar_reasons.iter().any(|b| b.contains("§ 7421(a)")));
        assert!(r.bar_reasons.iter().any(|b| b.contains("prong 1")));
        assert!(r.bar_reasons.iter().any(|b| b.contains("prong 2")));
    }

    #[test]
    fn statutory_exception_short_circuits_enochs_failure_reasons() {
        let mut i = restrain_assessment_base();
        i.statutory_exception = StatutoryException::Section6213a;
        let r = check(&i);
        assert!(r.bar_reasons.is_empty());
    }

    #[test]
    fn assessment_and_collection_engage_aia_bar_invariant() {
        for purpose in [
            SuitPurpose::RestrainAssessment,
            SuitPurpose::RestrainCollection,
        ] {
            let mut i = restrain_assessment_base();
            i.suit_purpose = purpose;
            let r = check(&i);
            assert!(r.aia_bar_engaged);
        }

        for purpose in [
            SuitPurpose::RegulatoryPreEnforcement,
            SuitPurpose::RefundAfterPayment,
        ] {
            let mut i = restrain_assessment_base();
            i.suit_purpose = purpose;
            let r = check(&i);
            assert!(!r.aia_bar_engaged);
        }
    }

    #[test]
    fn section_6232c_partnership_adjustment_permits_suit() {
        let mut i = restrain_assessment_base();
        i.statutory_exception = StatutoryException::Section6232c;
        let r = check(&i);
        assert!(r.suit_permitted);
    }

    #[test]
    fn section_7436_employment_status_permits_suit() {
        let mut i = restrain_assessment_base();
        i.statutory_exception = StatutoryException::Section7436;
        let r = check(&i);
        assert!(r.suit_permitted);
    }
}
