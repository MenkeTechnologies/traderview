//! IRC § 7216 — Disclosure or use of information by preparers
//! of returns. Criminal MISDEMEANOR (1-year imprisonment cap)
//! for tax return preparers who knowingly or recklessly disclose
//! or use tax return information for a purpose other than
//! preparing a tax return. Pairs with `section_6713` (civil
//! penalty $250 per prohibited disclosure, $10K annual cap) for
//! the same conduct. Trader-relevant: when broker, CPA, or tax
//! preparer discloses trader's return information without
//! authorization.
//!
//! **§ 7216(a) — Criminal offense**. Any person engaged in the
//! business of preparing returns who KNOWINGLY OR RECKLESSLY
//! discloses or uses tax return information for any purpose
//! other than preparing the return.
//!
//! **§ 7216(b) — Exceptions**:
//!
//! - TAXPAYER CONSENT obtained per regulations
//! - DISCLOSURES authorized by other Title 26 provisions
//! - DISCLOSURES authorized by Treasury regulations (26 CFR
//!   § 301.7216-2 lists specific permissible non-consent
//!   disclosures)
//! - USE for AUXILIARY tax-return-preparation purposes (computing
//!   estimated payments, etc.)
//!
//! **Consent regulations — 26 CFR § 301.7216-3**. Taxpayer
//! consent must satisfy specific format and content requirements
//! per Rev. Proc. 2013-14, including:
//!
//! - WRITTEN consent (electronic permitted)
//! - SIGNED BEFORE the disclosure or use
//! - Specifies the SPECIFIC RECIPIENT of the information
//! - Specifies the PURPOSE for which information will be used
//! - DURATION of the consent
//! - PROMINENT and SEPARATE from other documents
//!
//! Consent invalid if extracted by inadequate notice or contrary
//! to taxpayer interests.
//!
//! **Penalties**:
//!
//! - § 7216(a) base: imprisonment up to **1 YEAR (misdemeanor)**
//!   + fine up to $1,000 + costs of prosecution.
//! - 18 U.S.C. § 3571 supersedes original fine to $100,000
//!   individual / $200,000 corporation for class A misdemeanor.
//! - **Identity-theft enhancement**: monetary penalty up to
//!   $100,000 (separate from § 3571) when disclosure or use is
//!   in connection with a crime involving identity theft.
//!
//! **§ 6713 civil penalty** (paired with § 7216 same conduct):
//!
//! - $250 per prohibited disclosure or use
//! - $10,000 ANNUAL CAP per preparer per calendar year
//! - No-fault civil — does not require knowing or reckless
//!   conduct (lower standard than § 7216)
//!
//! **§ 6531 criminal SOL** — general 3-year SOL applies to
//! § 7216 (not extended 6-year as for § 7201 / § 7203 /
//! § 7206(1)-(4) / § 7207).
//!
//! **"Tax return information" broadly defined** — includes any
//! information furnished for, or in connection with, the
//! preparation of any tax return. Reaches financial records,
//! brokerage statements, K-1s, payroll information, source
//! documents.
//!
//! Citations: IRC § 7216(a) criminal offense; § 7216(b)
//! exceptions; § 6713(a) civil penalty + $10,000 annual cap;
//! 26 CFR § 301.7216-1 (definitions + general rules); 26 CFR
//! § 301.7216-2 (permissible disclosures and uses without
//! consent); 26 CFR § 301.7216-3 (consent requirements);
//! Rev. Proc. 2013-14 (standard consent form requirements);
//! 18 U.S.C. § 3571 (Criminal Fines Improvement Act); § 6531
//! (general 3-year criminal SOL); IRM 25.5.1 + IRM 4.10
//! (preparer penalty procedural manuals).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    Individual,
    Corporation,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section7216Input {
    pub entity_type: EntityType,
    /// Whether the preparer disclosed or used tax return
    /// information.
    pub preparer_disclosed_or_used_return_info: bool,
    /// Whether the mental state was KNOWING OR RECKLESS
    /// (required for § 7216(a) criminal liability; not required
    /// for § 6713 civil).
    pub disclosure_was_knowing_or_reckless: bool,
    /// Whether the purpose was OTHER THAN preparing the tax
    /// return (preparation-purpose disclosures are permitted).
    pub purpose_other_than_return_preparation: bool,
    /// Whether the taxpayer consented per regulations.
    pub taxpayer_consent_obtained: bool,
    /// Whether the consent complies with 26 CFR § 301.7216-3 +
    /// Rev. Proc. 2013-14 requirements (written + signed before
    /// disclosure + specific recipient + specific purpose +
    /// duration + prominent and separate).
    pub consent_complied_with_regs_301_7216_3: bool,
    /// Whether the disclosure or use is authorized by 26 CFR
    /// § 301.7216-2 permissible non-consent provisions.
    pub authorized_by_301_7216_2_non_consent: bool,
    /// Whether the disclosure is in connection with a crime
    /// involving identity theft (triggers $100,000 enhanced
    /// penalty).
    pub disclosure_involves_identity_theft: bool,
    /// Number of distinct prohibited disclosures or uses (for
    /// § 6713 civil penalty calculation at $250 each).
    pub number_of_prohibited_disclosures: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section7216Result {
    pub criminal_prosecution_authorized: bool,
    pub maximum_imprisonment_years: u32,
    pub maximum_fine_cents: i64,
    pub civil_penalty_authorized: bool,
    pub civil_penalty_amount_cents: i64,
    pub consent_exception_engaged: bool,
    pub non_consent_authorized_exception_engaged: bool,
    pub identity_theft_enhancement_engaged: bool,
    pub criminal_sol_years: u32,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section7216Input) -> Section7216Result {
    let mut notes: Vec<String> = Vec::new();

    notes.push(
        "§ 7216(a) — criminal MISDEMEANOR (1-year imprisonment cap) for preparer who KNOWINGLY OR RECKLESSLY discloses or uses tax return info for purpose other than preparing return"
            .to_string(),
    );

    notes.push(
        "18 U.S.C. § 3571 (Criminal Fines Improvement Act) — supersedes § 7216 original $1,000 fine cap to $100,000 individual / $200,000 corporation for class A misdemeanor"
            .to_string(),
    );

    let consent_engaged =
        input.taxpayer_consent_obtained && input.consent_complied_with_regs_301_7216_3;
    let non_consent_engaged = input.authorized_by_301_7216_2_non_consent;
    let any_exception_engaged = consent_engaged || non_consent_engaged;

    if input.taxpayer_consent_obtained {
        if consent_engaged {
            notes.push(
                "§ 7216(b) + 26 CFR § 301.7216-3 + Rev. Proc. 2013-14 — taxpayer consent obtained and complies with regulations (written + signed before disclosure + specific recipient + specific purpose + duration + prominent and separate); exception ENGAGED"
                    .to_string(),
            );
        } else {
            notes.push(
                "§ 7216(b) + 26 CFR § 301.7216-3 — taxpayer consent obtained but does NOT comply with regulations; consent invalid; exception NOT engaged"
                    .to_string(),
            );
        }
    }

    if non_consent_engaged {
        notes.push(
            "26 CFR § 301.7216-2 — permissible non-consent disclosures + uses include: court orders, IRS auditor inquiries, peer review, quality / peer reviews, malpractice insurance, payment processing for preparer compensation"
                .to_string(),
        );
    }

    let criminal_engaged = input.preparer_disclosed_or_used_return_info
        && input.disclosure_was_knowing_or_reckless
        && input.purpose_other_than_return_preparation
        && !any_exception_engaged;

    let civil_engaged = input.preparer_disclosed_or_used_return_info
        && input.purpose_other_than_return_preparation
        && !any_exception_engaged;

    let max_fine_base = match input.entity_type {
        EntityType::Individual => 10_000_000i64,
        EntityType::Corporation => 20_000_000i64,
    };

    let max_fine = if input.disclosure_involves_identity_theft {
        max_fine_base.max(10_000_000)
    } else {
        max_fine_base
    };

    if input.disclosure_involves_identity_theft {
        notes.push(
            "§ 7216 identity-theft enhancement — monetary penalty up to $100,000 when disclosure or use is in connection with a crime involving identity theft (separate from 18 U.S.C. § 3571 supersession)"
                .to_string(),
        );
    }

    let civil_per_disclosure = 25_000i64;
    let civil_annual_cap = 1_000_000i64;
    let raw_civil =
        (input.number_of_prohibited_disclosures as i64).saturating_mul(civil_per_disclosure);
    let civil_penalty = if civil_engaged {
        raw_civil.min(civil_annual_cap)
    } else {
        0
    };

    if civil_engaged {
        notes.push(format!(
            "§ 6713(a) civil penalty — $250 per prohibited disclosure × {} disclosures = ${} (capped at $10,000 annual = ${}); no-fault civil (does not require knowing or reckless conduct)",
            input.number_of_prohibited_disclosures,
            raw_civil / 100,
            civil_penalty / 100
        ));
    }

    notes.push(
        "§ 6531 general 3-year criminal SOL applies to § 7216 (not extended 6-year as for § 7201 / § 7203 / § 7206(1)-(4) / § 7207)"
            .to_string(),
    );

    notes.push(
        "'tax return information' broadly defined — includes any information furnished for or in connection with preparation of any tax return; reaches financial records + brokerage statements + K-1s + payroll information + source documents"
            .to_string(),
    );

    if criminal_engaged {
        notes.push(
            "§ 7216(a) — criminal misdemeanor prosecution AUTHORIZED (1-year imprisonment + fine + costs of prosecution)"
                .to_string(),
        );
    }

    notes.push("IRM 25.5.1 + IRM 4.10 — preparer penalty procedural manuals".to_string());

    Section7216Result {
        criminal_prosecution_authorized: criminal_engaged,
        maximum_imprisonment_years: 1,
        maximum_fine_cents: max_fine,
        civil_penalty_authorized: civil_engaged,
        civil_penalty_amount_cents: civil_penalty,
        consent_exception_engaged: consent_engaged,
        non_consent_authorized_exception_engaged: non_consent_engaged,
        identity_theft_enhancement_engaged: input.disclosure_involves_identity_theft,
        criminal_sol_years: 3,
        citation: "IRC §§ 7216(a), 7216(b), 6713(a), 6531; 26 CFR §§ 301.7216-1, 301.7216-2, 301.7216-3; Rev. Proc. 2013-14; 18 U.S.C. § 3571 (Criminal Fines Improvement Act); IRM 25.5.1; IRM 4.10",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full_violation() -> Section7216Input {
        Section7216Input {
            entity_type: EntityType::Individual,
            preparer_disclosed_or_used_return_info: true,
            disclosure_was_knowing_or_reckless: true,
            purpose_other_than_return_preparation: true,
            taxpayer_consent_obtained: false,
            consent_complied_with_regs_301_7216_3: false,
            authorized_by_301_7216_2_non_consent: false,
            disclosure_involves_identity_theft: false,
            number_of_prohibited_disclosures: 5,
        }
    }

    #[test]
    fn full_violation_criminal_and_civil_authorized() {
        let r = check(&full_violation());
        assert!(r.criminal_prosecution_authorized);
        assert!(r.civil_penalty_authorized);
        assert_eq!(r.maximum_imprisonment_years, 1);
        assert_eq!(r.civil_penalty_amount_cents, 5 * 25_000);
    }

    #[test]
    fn individual_max_fine_100k() {
        let r = check(&full_violation());
        assert_eq!(r.maximum_fine_cents, 10_000_000);
    }

    #[test]
    fn corporation_max_fine_200k() {
        let mut i = full_violation();
        i.entity_type = EntityType::Corporation;
        let r = check(&i);
        assert_eq!(r.maximum_fine_cents, 20_000_000);
    }

    #[test]
    fn no_disclosure_no_violation() {
        let mut i = full_violation();
        i.preparer_disclosed_or_used_return_info = false;
        let r = check(&i);
        assert!(!r.criminal_prosecution_authorized);
        assert!(!r.civil_penalty_authorized);
    }

    #[test]
    fn no_knowing_or_reckless_no_criminal_but_civil_still_engages() {
        let mut i = full_violation();
        i.disclosure_was_knowing_or_reckless = false;
        let r = check(&i);
        assert!(!r.criminal_prosecution_authorized);
        assert!(r.civil_penalty_authorized);
        assert_eq!(r.civil_penalty_amount_cents, 5 * 25_000);
    }

    #[test]
    fn preparation_purpose_no_violation() {
        let mut i = full_violation();
        i.purpose_other_than_return_preparation = false;
        let r = check(&i);
        assert!(!r.criminal_prosecution_authorized);
        assert!(!r.civil_penalty_authorized);
    }

    #[test]
    fn valid_consent_defeats_both_criminal_and_civil() {
        let mut i = full_violation();
        i.taxpayer_consent_obtained = true;
        i.consent_complied_with_regs_301_7216_3 = true;
        let r = check(&i);
        assert!(!r.criminal_prosecution_authorized);
        assert!(!r.civil_penalty_authorized);
        assert!(r.consent_exception_engaged);
    }

    #[test]
    fn invalid_consent_does_not_defeat() {
        let mut i = full_violation();
        i.taxpayer_consent_obtained = true;
        i.consent_complied_with_regs_301_7216_3 = false;
        let r = check(&i);
        assert!(r.criminal_prosecution_authorized);
        assert!(r.civil_penalty_authorized);
        assert!(!r.consent_exception_engaged);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("does NOT comply with regulations")));
    }

    #[test]
    fn non_consent_authorized_exception_defeats() {
        let mut i = full_violation();
        i.authorized_by_301_7216_2_non_consent = true;
        let r = check(&i);
        assert!(!r.criminal_prosecution_authorized);
        assert!(!r.civil_penalty_authorized);
        assert!(r.non_consent_authorized_exception_engaged);
    }

    #[test]
    fn identity_theft_enhancement_engaged() {
        let mut i = full_violation();
        i.disclosure_involves_identity_theft = true;
        let r = check(&i);
        assert!(r.identity_theft_enhancement_engaged);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("identity-theft enhancement") && n.contains("$100,000")));
    }

    #[test]
    fn civil_penalty_calculation_5_disclosures() {
        let r = check(&full_violation());
        assert_eq!(r.civil_penalty_amount_cents, 5 * 25_000);
    }

    #[test]
    fn civil_penalty_capped_at_10000_annual() {
        let mut i = full_violation();
        i.number_of_prohibited_disclosures = 100;
        let r = check(&i);
        assert_eq!(r.civil_penalty_amount_cents, 1_000_000);
    }

    #[test]
    fn civil_penalty_at_40_disclosures_at_cap() {
        let mut i = full_violation();
        i.number_of_prohibited_disclosures = 40;
        let r = check(&i);
        assert_eq!(r.civil_penalty_amount_cents, 1_000_000);
    }

    #[test]
    fn civil_penalty_at_39_disclosures_below_cap() {
        let mut i = full_violation();
        i.number_of_prohibited_disclosures = 39;
        let r = check(&i);
        assert_eq!(r.civil_penalty_amount_cents, 39 * 25_000);
    }

    #[test]
    fn civil_penalty_zero_when_no_violation() {
        let mut i = full_violation();
        i.preparer_disclosed_or_used_return_info = false;
        let r = check(&i);
        assert_eq!(r.civil_penalty_amount_cents, 0);
    }

    #[test]
    fn criminal_sol_3_years_general() {
        let r = check(&full_violation());
        assert_eq!(r.criminal_sol_years, 3);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6531 general 3-year criminal SOL")));
    }

    #[test]
    fn consent_regulations_note_describes_format_requirements() {
        let mut i = full_violation();
        i.taxpayer_consent_obtained = true;
        i.consent_complied_with_regs_301_7216_3 = true;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("§ 301.7216-3")
            && n.contains("Rev. Proc. 2013-14")
            && n.contains("written")
            && n.contains("prominent")));
    }

    #[test]
    fn non_consent_disclosure_note_lists_categories() {
        let mut i = full_violation();
        i.authorized_by_301_7216_2_non_consent = true;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("§ 301.7216-2")
            && n.contains("court orders")
            && n.contains("malpractice insurance")));
    }

    #[test]
    fn tax_return_information_broad_definition_note() {
        let r = check(&full_violation());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("brokerage statements") && n.contains("K-1s")));
    }

    #[test]
    fn cfia_18_usc_3571_note_present() {
        let r = check(&full_violation());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("18 U.S.C. § 3571") && n.contains("$100,000")));
    }

    #[test]
    fn irm_notes_present() {
        let r = check(&full_violation());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("IRM 25.5.1") && n.contains("IRM 4.10")));
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&full_violation());
        assert!(r.citation.contains("§§ 7216(a), 7216(b)"));
        assert!(r.citation.contains("6713(a)"));
        assert!(r.citation.contains("6531"));
        assert!(r.citation.contains("§§ 301.7216-1, 301.7216-2, 301.7216-3"));
        assert!(r.citation.contains("Rev. Proc. 2013-14"));
        assert!(r.citation.contains("18 U.S.C. § 3571"));
        assert!(r.citation.contains("IRM 25.5.1"));
        assert!(r.citation.contains("IRM 4.10"));
    }

    #[test]
    fn criminal_authorized_note_describes_misdemeanor() {
        let r = check(&full_violation());
        assert!(r.notes.iter().any(|n| n.contains("§ 7216(a)")
            && n.contains("misdemeanor prosecution AUTHORIZED")
            && n.contains("1-year")));
    }

    #[test]
    fn consent_truth_table() {
        for obtained in [false, true] {
            for compliant in [false, true] {
                let mut i = full_violation();
                i.taxpayer_consent_obtained = obtained;
                i.consent_complied_with_regs_301_7216_3 = compliant;
                let r = check(&i);
                let engaged = obtained && compliant;
                assert_eq!(r.consent_exception_engaged, engaged);
                assert_eq!(r.criminal_prosecution_authorized, !engaged);
            }
        }
    }

    #[test]
    fn civil_vs_criminal_no_fault_distinction() {
        let mut i = full_violation();
        i.disclosure_was_knowing_or_reckless = false;
        let r = check(&i);
        assert!(!r.criminal_prosecution_authorized);
        assert!(r.civil_penalty_authorized);
    }

    #[test]
    fn either_exception_pathway_defeats_violation() {
        let mut i_consent = full_violation();
        i_consent.taxpayer_consent_obtained = true;
        i_consent.consent_complied_with_regs_301_7216_3 = true;
        let r_consent = check(&i_consent);
        assert!(!r_consent.criminal_prosecution_authorized);

        let mut i_non_consent = full_violation();
        i_non_consent.authorized_by_301_7216_2_non_consent = true;
        let r_non_consent = check(&i_non_consent);
        assert!(!r_non_consent.criminal_prosecution_authorized);
    }

    #[test]
    fn imprisonment_always_1_year_misdemeanor() {
        let r = check(&full_violation());
        assert_eq!(r.maximum_imprisonment_years, 1);
    }

    #[test]
    fn civil_penalty_proportional_until_cap() {
        for disclosures in [1u32, 5, 10, 20, 30, 39, 40, 50, 100] {
            let mut i = full_violation();
            i.number_of_prohibited_disclosures = disclosures;
            let r = check(&i);
            let expected = ((disclosures as i64) * 25_000).min(1_000_000);
            assert_eq!(r.civil_penalty_amount_cents, expected);
        }
    }

    #[test]
    fn no_violation_no_civil_penalty() {
        let mut i = full_violation();
        i.purpose_other_than_return_preparation = false;
        let r = check(&i);
        assert_eq!(r.civil_penalty_amount_cents, 0);
    }
}
