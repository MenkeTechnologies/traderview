//! IRC § 6713 — Disclosure or use of information by
//! preparers of returns. Civil penalty companion to § 7216
//! criminal penalty for unauthorized disclosure or use of
//! tax return information by return preparers. Trader-
//! relevant because traders share extensive financial
//! information with preparers (1099-Bs + 1099-Ks + K-1s +
//! § 475(f) M2M election history + cost basis records +
//! § 1091 wash sale tracking + § 1256 60/40 reporting +
//! § 988 currency reporting + § 6038D foreign asset
//! reporting). § 6713 protects against preparer monetizing,
//! sharing, or otherwise using that data without taxpayer
//! consent. Companion to § 7216 (criminal disclosure
//! penalty — already shipped), § 7525 (FATP privilege), §
//! 6694 (preparer substantive penalty), § 6695 (preparer
//! procedural penalty), Circular 230 § 10.50.
//!
//! **§ 6713(a) Imposition of penalty** — any person who is
//! engaged in the business of preparing, or providing
//! services in connection with the preparation of, returns
//! of tax imposed by chapter 1, or any person who for
//! compensation prepares any such return for any other
//! person, AND who:
//! 1. **Discloses any information** furnished to him for, or
//!    in connection with, the preparation of any such
//!    return; OR
//! 2. **Uses any such information for any purpose other than
//!    to prepare, or assist in preparing, any such return**;
//!
//! SHALL pay a penalty of **$250 for each such disclosure
//! or use**, but the total amount imposed on any person for
//! any calendar year shall not exceed **$10,000**.
//!
//! **§ 6713(b) Exceptions** — the rules of § 7216(b) apply
//! to § 6713; permissible disclosures without consent
//! include:
//! 1. Disclosure pursuant to **court order or subpoena**;
//! 2. Disclosure to **another preparer in the firm**;
//! 3. Disclosure to **another firm assisting in preparation**
//!    (e.g., e-filing service provider);
//! 4. Disclosure for **bookkeeping services**;
//! 5. Disclosure for **quality or peer review** purposes;
//! 6. Disclosure for **professional liability insurance**;
//! 7. Disclosure to **federal/state/local tax authorities**
//!    pursuant to lawful investigation;
//! 8. Disclosure required by **other federal law**.
//!
//! **§ 6713 vs § 7216 distinction**:
//! - **§ 6713 civil penalty**: strict liability — does NOT
//!   require KNOWING OR RECKLESS conduct; $250/disclosure
//!   penalty + $10,000 annual cap.
//! - **§ 7216 criminal penalty**: requires **KNOWING OR
//!   RECKLESS** conduct; misdemeanor; up to **1 year
//!   imprisonment** + **$1,000 fine** + costs of
//!   prosecution.
//! - Both penalties may apply concurrently to the same
//!   disclosure (§ 6713(b) coordination via § 7216(b)
//!   exceptions).
//!
//! **Common consent forms** — preparer must obtain
//! taxpayer's signed and dated **written consent** before
//! using tax return info for (a) cross-selling additional
//! services, (b) outsourcing to overseas preparer, (c)
//! sharing with third-party marketing services, (d) any
//! other non-preparation use. AICPA + IRS Rev. Proc. 2013-
//! 14 provide sample consent language.
//!
//! Citations: 26 USC § 6713(a)-(b); 26 USC § 7216 (criminal
//! companion); 26 CFR § 301.7216-1 + § 301.7216-2 + §
//! 301.7216-3 (consent forms) + § 301.7216-4 (exceptions);
//! Rev. Proc. 2013-14 (sample consent forms); IRM 20.1.6
//! (Preparer and Promoter Penalties); Circular 230 § 10.50;
//! § 7525 FATP privilege; § 6694 preparer substantive
//! penalty; § 6695 preparer procedural penalty; AICPA
//! Section 7216 Guidance.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DisclosureExceptionCategory {
    /// § 301.7216-2 court order or subpoena.
    CourtOrderOrSubpoena,
    /// § 301.7216-2 disclosure to another preparer in the
    /// firm.
    PreparerInFirm,
    /// § 301.7216-2 disclosure to firm assisting in
    /// preparation (e-filing service).
    AssistingFirm,
    /// § 301.7216-2 bookkeeping services.
    BookkeepingServices,
    /// § 301.7216-2 quality or peer review.
    QualityOrPeerReview,
    /// § 301.7216-2 professional liability insurance.
    ProfessionalLiabilityInsurance,
    /// § 301.7216-2 federal/state/local tax authority
    /// investigation.
    TaxAuthorityInvestigation,
    /// § 301.7216-2 disclosure required by other federal
    /// law.
    OtherFederalLaw,
    /// Taxpayer's signed and dated written consent obtained.
    TaxpayerWrittenConsent,
    /// No exception applies.
    None,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6713Input {
    /// Number of unauthorized disclosures or uses by the
    /// preparer.
    pub disclosure_count: u32,
    /// Exception category (if any) that may shield
    /// disclosure.
    pub exception_category: DisclosureExceptionCategory,
    /// Whether disclosure was to a non-preparation purpose
    /// (cross-selling, marketing, etc.) without consent.
    pub non_preparation_use_without_consent: bool,
    /// Whether § 7216 criminal penalty was also imposed (may
    /// apply concurrently).
    pub section_7216_criminal_imposed: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6713Result {
    pub penalty_imposed: bool,
    pub exception_applies: bool,
    pub per_disclosure_penalty_cents: u64,
    pub annual_cap_cents: u64,
    pub total_penalty_cents: u64,
    pub annual_cap_engaged: bool,
    pub strict_liability_applies: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section6713Input) -> Section6713Result {
    let mut failure_reasons: Vec<String> = Vec::new();

    let exception_applies = !matches!(
        input.exception_category,
        DisclosureExceptionCategory::None
    );

    const PER_DISCLOSURE_CENTS: u64 = 25_000;
    const ANNUAL_CAP_CENTS: u64 = 1_000_000;

    let imposed = !exception_applies && input.disclosure_count > 0;

    let uncapped = PER_DISCLOSURE_CENTS.saturating_mul(input.disclosure_count as u64);
    let total = uncapped.min(ANNUAL_CAP_CENTS);
    let cap_engaged = imposed && uncapped > ANNUAL_CAP_CENTS;

    let total_penalty = if imposed { total } else { 0 };

    if imposed && input.non_preparation_use_without_consent {
        failure_reasons.push(
            "26 USC § 6713(a)(2) — preparer may not use information furnished in connection with preparation of return for any purpose other than to prepare or assist in preparing the return; written consent required for non-preparation uses (cross-selling + outsourcing + third-party marketing)".to_string(),
        );
    } else if imposed {
        failure_reasons.push(
            "26 USC § 6713(a)(1) — preparer may not disclose information furnished in connection with preparation of return; no exception applies".to_string(),
        );
    }

    let notes: Vec<String> = vec![
        "26 USC § 6713(a) — any person engaged in business of preparing returns who (1) discloses any information furnished for preparation OR (2) uses such information for any purpose other than to prepare the return SHALL pay penalty of $250 per disclosure/use; total $10,000/calendar year cap".to_string(),
        "26 USC § 6713(b) — § 7216(b) exceptions apply to § 6713; permissible disclosures without consent: court order/subpoena + preparer in firm + assisting firm + bookkeeping + quality/peer review + professional liability insurance + tax authority investigation + other federal law".to_string(),
        "§ 6713 vs § 7216 distinction — § 6713 civil penalty is STRICT LIABILITY (no knowing/reckless requirement); § 7216 criminal penalty requires KNOWING OR RECKLESS conduct (misdemeanor + up to 1 year imprisonment + $1,000 fine + costs of prosecution)".to_string(),
        "Both penalties may apply concurrently to the same disclosure — § 6713 civil + § 7216 criminal stack via § 6713(b) coordination with § 7216(b) exceptions".to_string(),
        "Common consent forms — preparer must obtain taxpayer's signed and dated WRITTEN CONSENT before using tax return info for (a) cross-selling additional services, (b) outsourcing to overseas preparer, (c) sharing with third-party marketing, (d) any other non-preparation use; AICPA + IRS Rev. Proc. 2013-14 provide sample consent language".to_string(),
        "Trader relevance — § 6713 protects against preparer monetizing/sharing trader financial information (1099-Bs + 1099-Ks + K-1s + § 475(f) M2M election history + cost basis records + § 1091 wash sale tracking + § 1256 60/40 reporting + § 988 currency reporting + § 6038D foreign asset reporting)".to_string(),
        "IRM 20.1.6 (Preparer and Promoter Penalties) — internal IRS procedural guidance on § 6713 + § 7216 + § 6694 + § 6695 + § 6700 + § 6701 preparer/promoter penalty cluster".to_string(),
    ];

    Section6713Result {
        penalty_imposed: imposed,
        exception_applies,
        per_disclosure_penalty_cents: PER_DISCLOSURE_CENTS,
        annual_cap_cents: ANNUAL_CAP_CENTS,
        total_penalty_cents: total_penalty,
        annual_cap_engaged: cap_engaged,
        strict_liability_applies: true,
        failure_reasons,
        citation: "26 USC § 6713(a)-(b); 26 USC § 7216; 26 CFR § 301.7216-1 + § 301.7216-2 + § 301.7216-3 + § 301.7216-4; Rev. Proc. 2013-14; IRM 20.1.6; Circular 230 § 10.50; § 7525; § 6694; § 6695",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_base() -> Section6713Input {
        Section6713Input {
            disclosure_count: 1,
            exception_category: DisclosureExceptionCategory::None,
            non_preparation_use_without_consent: true,
            section_7216_criminal_imposed: false,
        }
    }

    #[test]
    fn single_disclosure_without_exception_imposed() {
        let r = check(&valid_base());
        assert!(r.penalty_imposed);
        assert_eq!(r.total_penalty_cents, 25_000);
        assert!(!r.exception_applies);
    }

    #[test]
    fn no_disclosure_no_penalty() {
        let mut i = valid_base();
        i.disclosure_count = 0;
        let r = check(&i);
        assert!(!r.penalty_imposed);
        assert_eq!(r.total_penalty_cents, 0);
    }

    #[test]
    fn ten_disclosures_2500_dollars() {
        let mut i = valid_base();
        i.disclosure_count = 10;
        let r = check(&i);
        assert_eq!(r.total_penalty_cents, 250_000);
        assert!(!r.annual_cap_engaged);
    }

    #[test]
    fn forty_disclosures_at_10k_cap() {
        let mut i = valid_base();
        i.disclosure_count = 40;
        let r = check(&i);
        assert_eq!(r.total_penalty_cents, 1_000_000);
        assert!(!r.annual_cap_engaged);
    }

    #[test]
    fn forty_one_disclosures_cap_engaged() {
        let mut i = valid_base();
        i.disclosure_count = 41;
        let r = check(&i);
        assert_eq!(r.total_penalty_cents, 1_000_000);
        assert!(r.annual_cap_engaged);
    }

    #[test]
    fn court_order_exception_blocks_penalty() {
        let mut i = valid_base();
        i.exception_category = DisclosureExceptionCategory::CourtOrderOrSubpoena;
        let r = check(&i);
        assert!(!r.penalty_imposed);
        assert!(r.exception_applies);
        assert_eq!(r.total_penalty_cents, 0);
    }

    #[test]
    fn preparer_in_firm_exception_blocks_penalty() {
        let mut i = valid_base();
        i.exception_category = DisclosureExceptionCategory::PreparerInFirm;
        let r = check(&i);
        assert!(!r.penalty_imposed);
        assert!(r.exception_applies);
    }

    #[test]
    fn assisting_firm_exception_blocks_penalty() {
        let mut i = valid_base();
        i.exception_category = DisclosureExceptionCategory::AssistingFirm;
        let r = check(&i);
        assert!(!r.penalty_imposed);
    }

    #[test]
    fn bookkeeping_services_exception_blocks_penalty() {
        let mut i = valid_base();
        i.exception_category = DisclosureExceptionCategory::BookkeepingServices;
        let r = check(&i);
        assert!(!r.penalty_imposed);
    }

    #[test]
    fn quality_or_peer_review_exception_blocks_penalty() {
        let mut i = valid_base();
        i.exception_category = DisclosureExceptionCategory::QualityOrPeerReview;
        let r = check(&i);
        assert!(!r.penalty_imposed);
    }

    #[test]
    fn professional_liability_insurance_exception_blocks_penalty() {
        let mut i = valid_base();
        i.exception_category = DisclosureExceptionCategory::ProfessionalLiabilityInsurance;
        let r = check(&i);
        assert!(!r.penalty_imposed);
    }

    #[test]
    fn tax_authority_investigation_exception_blocks_penalty() {
        let mut i = valid_base();
        i.exception_category = DisclosureExceptionCategory::TaxAuthorityInvestigation;
        let r = check(&i);
        assert!(!r.penalty_imposed);
    }

    #[test]
    fn other_federal_law_exception_blocks_penalty() {
        let mut i = valid_base();
        i.exception_category = DisclosureExceptionCategory::OtherFederalLaw;
        let r = check(&i);
        assert!(!r.penalty_imposed);
    }

    #[test]
    fn taxpayer_written_consent_exception_blocks_penalty() {
        let mut i = valid_base();
        i.exception_category = DisclosureExceptionCategory::TaxpayerWrittenConsent;
        let r = check(&i);
        assert!(!r.penalty_imposed);
    }

    #[test]
    fn strict_liability_applies_always() {
        let r = check(&valid_base());
        assert!(r.strict_liability_applies);
    }

    #[test]
    fn non_preparation_use_without_consent_failure_reason_present() {
        let r = check(&valid_base());
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6713(a)(2)") && f.contains("cross-selling")));
    }

    #[test]
    fn pure_disclosure_failure_reason_present_when_not_use() {
        let mut i = valid_base();
        i.non_preparation_use_without_consent = false;
        let r = check(&i);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6713(a)(1)") && f.contains("disclose information")));
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&valid_base());
        assert!(r.citation.contains("§ 6713(a)-(b)"));
        assert!(r.citation.contains("§ 7216"));
        assert!(r.citation.contains("§ 301.7216-1"));
        assert!(r.citation.contains("§ 301.7216-2"));
        assert!(r.citation.contains("§ 301.7216-3"));
        assert!(r.citation.contains("§ 301.7216-4"));
        assert!(r.citation.contains("Rev. Proc. 2013-14"));
        assert!(r.citation.contains("IRM 20.1.6"));
        assert!(r.citation.contains("Circular 230 § 10.50"));
        assert!(r.citation.contains("§ 7525"));
    }

    #[test]
    fn note_pins_subsection_a_250_dollar_per_disclosure_10k_cap() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6713(a)")
            && n.contains("$250 per disclosure")
            && n.contains("$10,000")));
    }

    #[test]
    fn note_pins_subsection_b_seven_exceptions() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6713(b)")
            && n.contains("court order")
            && n.contains("preparer in firm")
            && n.contains("bookkeeping")
            && n.contains("peer review")
            && n.contains("professional liability insurance")
            && n.contains("tax authority investigation")));
    }

    #[test]
    fn note_pins_6713_vs_7216_distinction() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6713 vs § 7216")
            && n.contains("STRICT LIABILITY")
            && n.contains("KNOWING OR RECKLESS")
            && n.contains("misdemeanor")
            && n.contains("1 year imprisonment")));
    }

    #[test]
    fn note_pins_concurrent_penalty_stacking() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("Both penalties may apply concurrently")));
    }

    #[test]
    fn note_pins_common_consent_forms() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("Rev. Proc. 2013-14")
            && n.contains("AICPA")
            && n.contains("cross-selling")
            && n.contains("outsourcing")));
    }

    #[test]
    fn note_pins_trader_relevance() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("1099-Bs")
            && n.contains("§ 475(f) M2M")
            && n.contains("§ 1091 wash sale")
            && n.contains("§ 6038D")));
    }

    #[test]
    fn note_pins_irm_20_1_6_preparer_cluster() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("IRM 20.1.6")
            && n.contains("§ 6694")
            && n.contains("§ 6695")
            && n.contains("§ 6700")
            && n.contains("§ 6701")));
    }

    #[test]
    fn exception_category_truth_table_ten_cells() {
        for (category, exp_applies) in [
            (DisclosureExceptionCategory::CourtOrderOrSubpoena, true),
            (DisclosureExceptionCategory::PreparerInFirm, true),
            (DisclosureExceptionCategory::AssistingFirm, true),
            (DisclosureExceptionCategory::BookkeepingServices, true),
            (DisclosureExceptionCategory::QualityOrPeerReview, true),
            (DisclosureExceptionCategory::ProfessionalLiabilityInsurance, true),
            (DisclosureExceptionCategory::TaxAuthorityInvestigation, true),
            (DisclosureExceptionCategory::OtherFederalLaw, true),
            (DisclosureExceptionCategory::TaxpayerWrittenConsent, true),
            (DisclosureExceptionCategory::None, false),
        ] {
            let mut i = valid_base();
            i.exception_category = category;
            let r = check(&i);
            assert_eq!(r.exception_applies, exp_applies);
            assert_eq!(r.penalty_imposed, !exp_applies && i.disclosure_count > 0);
        }
    }

    #[test]
    fn annual_cap_invariant_at_40_disclosures() {
        let mut i_40 = valid_base();
        i_40.disclosure_count = 40;
        let r_40 = check(&i_40);
        assert_eq!(r_40.total_penalty_cents, 1_000_000);
        assert!(!r_40.annual_cap_engaged);

        let mut i_41 = valid_base();
        i_41.disclosure_count = 41;
        let r_41 = check(&i_41);
        assert_eq!(r_41.total_penalty_cents, 1_000_000);
        assert!(r_41.annual_cap_engaged);
    }

    #[test]
    fn strict_liability_distinct_from_7216_knowing_invariant() {
        let r = check(&valid_base());
        assert!(r.strict_liability_applies);
        assert!(r.notes.iter().any(|n| n.contains("STRICT LIABILITY")
            && n.contains("KNOWING OR RECKLESS")));
    }

    #[test]
    fn defensive_max_u32_disclosure_count_caps_at_10k() {
        let mut i = valid_base();
        i.disclosure_count = u32::MAX;
        let r = check(&i);
        assert_eq!(r.total_penalty_cents, 1_000_000);
        assert!(r.annual_cap_engaged);
    }

    #[test]
    fn taxpayer_written_consent_blocks_even_for_marketing_invariant() {
        let mut i = valid_base();
        i.exception_category = DisclosureExceptionCategory::TaxpayerWrittenConsent;
        i.non_preparation_use_without_consent = true;
        let r = check(&i);
        assert!(!r.penalty_imposed);
    }
}
