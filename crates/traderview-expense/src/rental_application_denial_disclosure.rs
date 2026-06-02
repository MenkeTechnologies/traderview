//! Rental application denial reason written disclosure
//! compliance — when a trader-landlord rejects a tenant
//! applicant, what statutory written-disclosure obligations
//! attach? Trader-landlord operational concern: an
//! incomplete or untimely denial-reason disclosure exposes
//! landlord to statutory damages + applicant claim for
//! discriminatory denial + reputational risk.
//!
//! Distinct from siblings `adverse_action_notice` (FCRA-
//! specific notice for consumer report denials only),
//! `fair_chance_housing` (criminal-background screening
//! framework), `application_fees` (caps on application fee
//! itself), and `credit_check_authorization` (pre-screening
//! consent).
//!
//! **Four regimes**:
//!
//! **California — Cal. Civ. Code § 1950.6 + § 1786.40
//! (ICRAA Investigative Consumer Reporting Agencies Act)**:
//! - Written notice required to applicant when credit score
//!   or history was reason for denial.
//! - Specific written reason for denial required.
//! - Application screening fee may be retained if applicant
//!   doesn't meet established screening criteria.
//!
//! **New Jersey — Fair Chance in Housing Act + Fair Chance
//! Act (N.J.S.A. 46:8-52 et seq.)**:
//! - Must disclose IN WRITING before accepting application
//!   fee whether eligibility considers criminal history.
//! - Must give written statement allowing applicants to
//!   provide evidence of rehabilitation or mitigating
//!   factors.
//! - Specific written reason for criminal-history-based
//!   denial required.
//!
//! **NYC — Fair Chance for Housing Law (Local Law 24 of
//! 2023, effective January 1, 2025)**:
//! - Conditional offer + lookback + individualized
//!   assessment framework for criminal-history denials.
//! - Written notice required with specific reason and
//!   appeal rights.
//! - FARE Act (effective June 11, 2025) — broker fees
//!   prohibited from being charged to tenants.
//!
//! **Default — Federal FCRA § 615(a), 15 USC § 1681m**:
//! - Adverse action notice required when consumer report
//!   was basis for denial.
//! - Must include CRA contact info + dispute right + free-
//!   copy right + statement that landlord made decision.
//! - LESS stringent than CA / NJ / NYC.
//!
//! Citations: Cal. Civ. Code §§ 1950.6, 1786.40 (ICRAA);
//! N.J.S.A. 46:8-52 et seq. (NJ Fair Chance in Housing
//! Act); NYC Local Law 24 of 2023 (eff. January 1, 2025);
//! NYC FARE Act (eff. June 11, 2025); 15 USC § 1681m (FCRA
//! § 615(a) adverse action notice).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    California,
    NewJersey,
    NewYorkCity,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DenialBasis {
    /// Denial based on credit score or history.
    CreditScoreOrHistory,
    /// Denial based on criminal background.
    CriminalBackground,
    /// Denial based on insufficient income.
    InsufficientIncome,
    /// Denial based on negative landlord references.
    NegativeLandlordReferences,
    /// Denial based on no clear reason / pretextual.
    Pretextual,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RentalApplicationDenialDisclosureInput {
    pub regime: Regime,
    pub denial_basis: DenialBasis,
    /// Whether written notice of denial was provided.
    pub written_notice_provided: bool,
    /// Whether specific reason for denial was disclosed.
    pub specific_reason_disclosed: bool,
    /// Whether pre-fee criminal-history disclosure was given
    /// (NJ Fair Chance Act).
    pub pre_fee_criminal_history_disclosure: bool,
    /// Whether applicant's right to provide rehabilitation
    /// evidence was disclosed (NJ + NYC).
    pub rehabilitation_evidence_right_disclosed: bool,
    /// Whether individualized assessment for criminal
    /// background was conducted (NYC FCHA + NJ).
    pub individualized_assessment_conducted: bool,
    /// Whether CRA contact info was provided (federal FCRA).
    pub cra_contact_info_provided: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RentalApplicationDenialDisclosureResult {
    pub disclosure_compliant: bool,
    pub statutory_protection_engaged: bool,
    pub pre_fee_disclosure_required: bool,
    pub individualized_assessment_required: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &RentalApplicationDenialDisclosureInput) -> RentalApplicationDenialDisclosureResult {
    match input.regime {
        Regime::California => check_ca(input),
        Regime::NewJersey => check_nj(input),
        Regime::NewYorkCity => check_nyc(input),
        Regime::Default => check_default(input),
    }
}

fn check_ca(
    input: &RentalApplicationDenialDisclosureInput,
) -> RentalApplicationDenialDisclosureResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Cal. Civ. Code § 1950.6 — landlord must notify applicant in writing when credit score or history was reason for denial; specific written reason for denial required"
            .to_string(),
        "Cal. Civ. Code § 1786.40 (ICRAA) — Investigative Consumer Reporting Agencies Act requires specific written disclosure when investigative consumer report is basis for denial"
            .to_string(),
    ];

    let credit_basis = matches!(
        input.denial_basis,
        DenialBasis::CreditScoreOrHistory
    );

    if credit_basis && !input.written_notice_provided {
        violations.push(
            "Cal. Civ. Code § 1950.6 — written notice required when credit score/history was reason for denial".to_string(),
        );
    }

    if !input.specific_reason_disclosed {
        violations.push(
            "Cal. Civ. Code §§ 1950.6 + 1786.40 (ICRAA) — specific written reason for denial required".to_string(),
        );
    }

    RentalApplicationDenialDisclosureResult {
        disclosure_compliant: violations.is_empty(),
        statutory_protection_engaged: true,
        pre_fee_disclosure_required: false,
        individualized_assessment_required: false,
        violations,
        citation: "Cal. Civ. Code §§ 1950.6, 1786.40 (ICRAA)",
        notes,
    }
}

fn check_nj(
    input: &RentalApplicationDenialDisclosureInput,
) -> RentalApplicationDenialDisclosureResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "N.J.S.A. 46:8-52 et seq. (NJ Fair Chance in Housing Act) — must disclose IN WRITING BEFORE accepting application fee whether eligibility considers criminal history"
            .to_string(),
        "N.J.S.A. 46:8-52 et seq. (NJ FCHA) — must give written statement allowing applicants to provide evidence of rehabilitation or mitigating factors; specific written reason for criminal-history-based denial required"
            .to_string(),
    ];

    let criminal_basis = matches!(
        input.denial_basis,
        DenialBasis::CriminalBackground
    );

    if criminal_basis && !input.pre_fee_criminal_history_disclosure {
        violations.push(
            "N.J.S.A. 46:8-52 et seq. — pre-fee written disclosure of criminal-history consideration required BEFORE accepting application fee".to_string(),
        );
    }

    if criminal_basis && !input.rehabilitation_evidence_right_disclosed {
        violations.push(
            "N.J.S.A. 46:8-52 et seq. — must inform applicant of right to submit evidence of rehabilitation or mitigating factors".to_string(),
        );
    }

    if criminal_basis && !input.individualized_assessment_conducted {
        violations.push(
            "N.J.S.A. 46:8-52 et seq. — individualized assessment of criminal history required (cannot apply blanket criminal-record bar)".to_string(),
        );
    }

    if !input.specific_reason_disclosed {
        violations.push(
            "N.J.S.A. 46:8-52 et seq. — specific written reason for denial required".to_string(),
        );
    }

    RentalApplicationDenialDisclosureResult {
        disclosure_compliant: violations.is_empty(),
        statutory_protection_engaged: true,
        pre_fee_disclosure_required: criminal_basis,
        individualized_assessment_required: criminal_basis,
        violations,
        citation: "N.J.S.A. 46:8-52 et seq. (NJ Fair Chance in Housing Act)",
        notes,
    }
}

fn check_nyc(
    input: &RentalApplicationDenialDisclosureInput,
) -> RentalApplicationDenialDisclosureResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "NYC Local Law 24 of 2023 (Fair Chance for Housing Law, effective January 1, 2025) — conditional offer + lookback + individualized assessment framework for criminal-history denials; written notice required with specific reason and appeal rights"
            .to_string(),
        "NYC FARE Act (effective June 11, 2025) — broker fees prohibited from being charged to tenants; landlords/agents must disclose other fees the tenant must pay in listings and rental agreements"
            .to_string(),
    ];

    let criminal_basis = matches!(
        input.denial_basis,
        DenialBasis::CriminalBackground
    );

    if criminal_basis && !input.individualized_assessment_conducted {
        violations.push(
            "NYC Local Law 24 of 2023 — individualized assessment of criminal history required; blanket criminal-record bar unlawful".to_string(),
        );
    }

    if criminal_basis && !input.rehabilitation_evidence_right_disclosed {
        violations.push(
            "NYC Local Law 24 of 2023 — applicant must be given opportunity to provide rehabilitation evidence".to_string(),
        );
    }

    if !input.written_notice_provided || !input.specific_reason_disclosed {
        violations.push(
            "NYC Local Law 24 of 2023 — written notice with specific reason for denial required".to_string(),
        );
    }

    RentalApplicationDenialDisclosureResult {
        disclosure_compliant: violations.is_empty(),
        statutory_protection_engaged: true,
        pre_fee_disclosure_required: false,
        individualized_assessment_required: criminal_basis,
        violations,
        citation: "NYC Local Law 24 of 2023 (Fair Chance for Housing Law); NYC FARE Act (eff. June 11, 2025)",
        notes,
    }
}

fn check_default(
    input: &RentalApplicationDenialDisclosureInput,
) -> RentalApplicationDenialDisclosureResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "default rule — FCRA § 615(a), 15 USC § 1681m — adverse action notice required when consumer report was basis for denial; must include CRA contact info + dispute right + free-copy right + statement that landlord made decision"
            .to_string(),
        "default rule — federal FCRA is less stringent than CA / NJ / NYC; many states have no specific written-reason mandate beyond federal FCRA disclosure"
            .to_string(),
    ];

    let credit_basis = matches!(
        input.denial_basis,
        DenialBasis::CreditScoreOrHistory
    );

    if credit_basis && !input.cra_contact_info_provided {
        violations.push(
            "FCRA § 615(a), 15 USC § 1681m — adverse action notice required when consumer report was basis for denial; must include CRA contact information".to_string(),
        );
    }

    RentalApplicationDenialDisclosureResult {
        disclosure_compliant: violations.is_empty(),
        statutory_protection_engaged: false,
        pre_fee_disclosure_required: false,
        individualized_assessment_required: false,
        violations,
        citation: "15 USC § 1681m (FCRA § 615(a) federal baseline)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ca_compliant() -> RentalApplicationDenialDisclosureInput {
        RentalApplicationDenialDisclosureInput {
            regime: Regime::California,
            denial_basis: DenialBasis::CreditScoreOrHistory,
            written_notice_provided: true,
            specific_reason_disclosed: true,
            pre_fee_criminal_history_disclosure: false,
            rehabilitation_evidence_right_disclosed: false,
            individualized_assessment_conducted: false,
            cra_contact_info_provided: false,
        }
    }

    fn nj_compliant() -> RentalApplicationDenialDisclosureInput {
        RentalApplicationDenialDisclosureInput {
            regime: Regime::NewJersey,
            denial_basis: DenialBasis::CriminalBackground,
            written_notice_provided: true,
            specific_reason_disclosed: true,
            pre_fee_criminal_history_disclosure: true,
            rehabilitation_evidence_right_disclosed: true,
            individualized_assessment_conducted: true,
            cra_contact_info_provided: false,
        }
    }

    fn nyc_compliant() -> RentalApplicationDenialDisclosureInput {
        RentalApplicationDenialDisclosureInput {
            regime: Regime::NewYorkCity,
            denial_basis: DenialBasis::CriminalBackground,
            written_notice_provided: true,
            specific_reason_disclosed: true,
            pre_fee_criminal_history_disclosure: false,
            rehabilitation_evidence_right_disclosed: true,
            individualized_assessment_conducted: true,
            cra_contact_info_provided: false,
        }
    }

    fn default_base() -> RentalApplicationDenialDisclosureInput {
        let mut i = ca_compliant();
        i.regime = Regime::Default;
        i.cra_contact_info_provided = true;
        i
    }

    #[test]
    fn ca_credit_denial_with_notice_compliant() {
        let r = check(&ca_compliant());
        assert!(r.disclosure_compliant);
        assert!(r.statutory_protection_engaged);
    }

    #[test]
    fn ca_credit_denial_without_written_notice_violates() {
        let mut i = ca_compliant();
        i.written_notice_provided = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 1950.6") && v.contains("written notice")));
    }

    #[test]
    fn ca_no_specific_reason_violates() {
        let mut i = ca_compliant();
        i.specific_reason_disclosed = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§§ 1950.6 + 1786.40") && v.contains("specific written reason")));
    }

    #[test]
    fn ca_citation_pins_authorities() {
        let r = check(&ca_compliant());
        assert!(r.citation.contains("§§ 1950.6, 1786.40"));
        assert!(r.citation.contains("ICRAA"));
    }

    #[test]
    fn nj_criminal_denial_compliant() {
        let r = check(&nj_compliant());
        assert!(r.disclosure_compliant);
        assert!(r.pre_fee_disclosure_required);
        assert!(r.individualized_assessment_required);
    }

    #[test]
    fn nj_no_pre_fee_disclosure_violates() {
        let mut i = nj_compliant();
        i.pre_fee_criminal_history_disclosure = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("46:8-52") && v.contains("BEFORE")));
    }

    #[test]
    fn nj_no_rehabilitation_right_disclosed_violates() {
        let mut i = nj_compliant();
        i.rehabilitation_evidence_right_disclosed = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("rehabilitation")));
    }

    #[test]
    fn nj_no_individualized_assessment_violates() {
        let mut i = nj_compliant();
        i.individualized_assessment_conducted = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("individualized assessment") && v.contains("blanket")));
    }

    #[test]
    fn nj_non_criminal_basis_no_pre_fee_required() {
        let mut i = nj_compliant();
        i.denial_basis = DenialBasis::CreditScoreOrHistory;
        i.pre_fee_criminal_history_disclosure = false;
        i.rehabilitation_evidence_right_disclosed = false;
        i.individualized_assessment_conducted = false;
        let r = check(&i);
        assert!(r.disclosure_compliant);
        assert!(!r.pre_fee_disclosure_required);
    }

    #[test]
    fn nj_citation_pins_fcha() {
        let r = check(&nj_compliant());
        assert!(r.citation.contains("46:8-52"));
        assert!(r.citation.contains("Fair Chance in Housing Act"));
    }

    #[test]
    fn nyc_criminal_denial_compliant() {
        let r = check(&nyc_compliant());
        assert!(r.disclosure_compliant);
        assert!(r.individualized_assessment_required);
    }

    #[test]
    fn nyc_no_individualized_assessment_violates() {
        let mut i = nyc_compliant();
        i.individualized_assessment_conducted = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("Local Law 24") && v.contains("blanket")));
    }

    #[test]
    fn nyc_no_rehabilitation_evidence_violates() {
        let mut i = nyc_compliant();
        i.rehabilitation_evidence_right_disclosed = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("rehabilitation evidence")));
    }

    #[test]
    fn nyc_no_written_notice_violates() {
        let mut i = nyc_compliant();
        i.written_notice_provided = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
    }

    #[test]
    fn nyc_citation_pins_local_law_24_and_fare_act() {
        let r = check(&nyc_compliant());
        assert!(r.citation.contains("Local Law 24 of 2023"));
        assert!(r.citation.contains("Fair Chance for Housing Law"));
        assert!(r.citation.contains("FARE Act"));
        assert!(r.citation.contains("June 11, 2025"));
    }

    #[test]
    fn default_credit_denial_with_cra_notice_compliant() {
        let r = check(&default_base());
        assert!(r.disclosure_compliant);
        assert!(!r.statutory_protection_engaged);
    }

    #[test]
    fn default_no_cra_contact_violates_fcra() {
        let mut i = default_base();
        i.cra_contact_info_provided = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("FCRA § 615(a)") && v.contains("§ 1681m") && v.contains("CRA")));
    }

    #[test]
    fn default_citation_pins_fcra_baseline() {
        let r = check(&default_base());
        assert!(r.citation.contains("§ 1681m"));
        assert!(r.citation.contains("FCRA § 615(a)"));
    }

    #[test]
    fn four_regimes_routed_correctly() {
        for regime in [
            Regime::California,
            Regime::NewJersey,
            Regime::NewYorkCity,
            Regime::Default,
        ] {
            let mut i = ca_compliant();
            i.regime = regime;
            i.cra_contact_info_provided = true;
            i.rehabilitation_evidence_right_disclosed = true;
            i.individualized_assessment_conducted = true;
            let r = check(&i);
            assert!(!r.citation.is_empty());
        }
    }

    #[test]
    fn nj_uniquely_requires_pre_fee_disclosure_invariant() {
        let r_nj = check(&nj_compliant());
        assert!(r_nj.pre_fee_disclosure_required);

        let r_ca = check(&ca_compliant());
        assert!(!r_ca.pre_fee_disclosure_required);

        let r_nyc = check(&nyc_compliant());
        assert!(!r_nyc.pre_fee_disclosure_required);

        let r_default = check(&default_base());
        assert!(!r_default.pre_fee_disclosure_required);
    }

    #[test]
    fn nj_and_nyc_uniquely_require_individualized_assessment_invariant() {
        let r_nj = check(&nj_compliant());
        assert!(r_nj.individualized_assessment_required);

        let r_nyc = check(&nyc_compliant());
        assert!(r_nyc.individualized_assessment_required);

        let r_ca = check(&ca_compliant());
        assert!(!r_ca.individualized_assessment_required);

        let r_default = check(&default_base());
        assert!(!r_default.individualized_assessment_required);
    }

    #[test]
    fn statutory_protection_engaged_for_ca_nj_nyc_only_invariant() {
        for regime in [Regime::California, Regime::NewJersey, Regime::NewYorkCity] {
            let mut i = ca_compliant();
            i.regime = regime;
            i.pre_fee_criminal_history_disclosure = true;
            i.rehabilitation_evidence_right_disclosed = true;
            i.individualized_assessment_conducted = true;
            i.denial_basis = DenialBasis::CreditScoreOrHistory;
            let r = check(&i);
            assert!(r.statutory_protection_engaged);
        }

        let r_default = check(&default_base());
        assert!(!r_default.statutory_protection_engaged);
    }

    #[test]
    fn ca_note_pins_icraa() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 1786.40")
            && n.contains("ICRAA")
            && n.contains("Investigative Consumer Reporting Agencies Act")));
    }

    #[test]
    fn nj_note_pins_before_accepting_fee() {
        let r = check(&nj_compliant());
        assert!(r.notes.iter().any(|n| n.contains("BEFORE")
            && n.contains("application fee")
            && n.contains("criminal history")));
    }

    #[test]
    fn nyc_note_pins_january_1_2025_and_june_11_2025() {
        let r = check(&nyc_compliant());
        assert!(r.notes.iter().any(|n| n.contains("January 1, 2025")
            && n.contains("conditional offer")
            && n.contains("appeal rights")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("FARE Act") && n.contains("June 11, 2025")));
    }

    #[test]
    fn default_note_pins_fcra_less_stringent() {
        let r = check(&default_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("FCRA") && n.contains("less stringent")));
    }

    #[test]
    fn denial_basis_truth_table_nj() {
        for (basis, exp_pre_fee_required) in [
            (DenialBasis::CriminalBackground, true),
            (DenialBasis::CreditScoreOrHistory, false),
            (DenialBasis::InsufficientIncome, false),
            (DenialBasis::NegativeLandlordReferences, false),
            (DenialBasis::Pretextual, false),
        ] {
            let mut i = nj_compliant();
            i.denial_basis = basis;
            i.pre_fee_criminal_history_disclosure = true;
            i.rehabilitation_evidence_right_disclosed = true;
            i.individualized_assessment_conducted = true;
            let r = check(&i);
            assert_eq!(r.pre_fee_disclosure_required, exp_pre_fee_required);
        }
    }
}
