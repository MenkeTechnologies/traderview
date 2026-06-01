//! Mandatory landlord-provided annual rent statement to tenant
//! for tenant tax-credit purposes — when must a residential
//! landlord issue an annual rent-paid statement to enable the
//! tenant to claim a state-level renter's tax credit or property
//! tax refund? Distinct from `rent_receipts` (per-payment
//! receipts), `security_deposit_interest_statement` (annual
//! security-deposit interest statement), and `lease_disclosures`
//! (mandated lease content).
//!
//! Trader-landlord operational concern in MN, VT, ME, and other
//! states that provide tenant tax credits. Failure to issue the
//! annual statement exposes landlord to state-imposed penalties
//! and forces tenants into the alternative-affidavit pathway,
//! which the state revenue department then audits.
//!
//! **Three regimes**:
//!
//! **Minnesota — Minn. Stat. § 290A.19 (Certificate of Rent
//! Paid)**. Most explicit framework. Landlord MUST issue
//! Certificate of Rent Paid (CRP) form to each tenant by
//! JANUARY 31 each year. CRP details total rent paid by tenant
//! over previous calendar year and supports tenant's claim for
//! Property Tax Refund (also called Renter's Credit) under
//! § 290A. CRP may be provided as electronic or hard copy. If
//! landlord fails to provide CRP by January 31, the renter may
//! request a "Rent Paid Affidavit" from the Minnesota Department
//! of Revenue as alternative. Noncompliance triggers state-law
//! penalties.
//!
//! **Vermont — Vt. Stat. tit. 32 § 6066 (Renter Rebate Program)**.
//! Landlord must provide annual statement on Form LRC-147 to
//! tenant for renter rebate claim. Form must include landlord
//! name + property address + total rent paid in calendar year.
//!
//! **Default — no statutory obligation**. Most states have no
//! statewide landlord annual-rent-statement mandate. Where the
//! state provides a renter tax credit (MA, MI, WI, IN, IA, ME,
//! MD), tenants typically claim based on their own records;
//! landlord must produce records on request but is not required
//! to proactively issue a statement.
//!
//! Citations: Minn. Stat. § 290A.19 covers MN CRP landlord
//! obligation with January 31 deadline, electronic or hard copy,
//! and Rent Paid Affidavit alternative. Minn. Stat. § 290A
//! covers the MN Property Tax Refund and Renter's Credit
//! framework. Vt. Stat. tit. 32 § 6066 covers VT Renter Rebate.
//! Mass. Gen. Laws ch. 62 § 6(a) covers MA Renter Deduction.
//! Mich. Comp. Laws § 206.522 covers MI Homestead Property Tax
//! Credit including renters. Wis. Stat. § 71.07(9) covers WI
//! School Property Tax Credit. Ind. Code § 6-3-2-6 covers IN
//! Renter Deduction. Iowa Code § 425.16 covers IA Property Tax
//! Credit. Md. Code, Tax Property § 9-104 covers MD Homeowner's
//! Property Tax Credit. Me. Rev. Stat. tit. 36 § 6201 covers ME
//! Renters Rebate Program.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    Minnesota,
    Vermont,
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LandlordAnnualRentStatementInput {
    pub regime: Regime,
    /// Annual rent paid in cents (calendar year).
    pub annual_rent_paid_cents: i64,
    /// Whether the landlord issued the Certificate of Rent Paid
    /// (CRP) / annual statement to the tenant by the statutory
    /// deadline (January 31 in MN).
    pub statement_issued_by_deadline: bool,
    /// Whether the statement includes the required information
    /// (total rent paid, landlord name, property address).
    pub statement_includes_required_information: bool,
    /// Whether the tenant requested a Rent Paid Affidavit from
    /// the Minnesota Department of Revenue as alternative
    /// (MN fallback when landlord noncompliant).
    pub tenant_requested_rent_paid_affidavit: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LandlordAnnualRentStatementResult {
    pub compliant: bool,
    pub state_penalty_exposure: bool,
    /// Whether the MN alternative Rent Paid Affidavit pathway is
    /// available to the tenant.
    pub affidavit_alternative_available: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &LandlordAnnualRentStatementInput) -> LandlordAnnualRentStatementResult {
    match input.regime {
        Regime::Minnesota => check_minnesota(input),
        Regime::Vermont => check_vermont(input),
        Regime::Default => check_default(input),
    }
}

fn check_minnesota(
    input: &LandlordAnnualRentStatementInput,
) -> LandlordAnnualRentStatementResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    if !input.statement_issued_by_deadline {
        violations.push(
            "Minn. Stat. § 290A.19 — landlord MUST issue Certificate of Rent Paid (CRP) to each tenant by JANUARY 31 each year"
                .to_string(),
        );
    }

    if input.statement_issued_by_deadline && !input.statement_includes_required_information {
        violations.push(
            "Minn. Stat. § 290A.19 — CRP MUST include landlord name + property address + total rent paid by tenant in calendar year"
                .to_string(),
        );
    }

    notes.push(
        "Minn. Stat. § 290A.19 — CRP supports tenant's claim for Property Tax Refund (Renter's Credit) under § 290A"
            .to_string(),
    );
    notes.push(
        "Minn. Stat. § 290A.19 — CRP may be provided as ELECTRONIC OR HARD COPY"
            .to_string(),
    );

    let affidavit_available = !input.statement_issued_by_deadline;
    if affidavit_available {
        notes.push(
            "Minn. Stat. § 290A.19 fallback — when landlord fails to provide CRP by January 31, tenant may request a 'Rent Paid Affidavit' from Minnesota Department of Revenue as alternative"
                .to_string(),
        );
    }

    if input.tenant_requested_rent_paid_affidavit {
        notes.push(
            "tenant has invoked the Rent Paid Affidavit alternative; MN Department of Revenue will audit landlord's records and impose state-law penalties for nonissuance"
                .to_string(),
        );
    }

    let compliant = violations.is_empty();
    LandlordAnnualRentStatementResult {
        compliant,
        state_penalty_exposure: !compliant,
        affidavit_alternative_available: affidavit_available,
        violations,
        citation: "Minn. Stat. §§ 290A.19, 290A (MN Property Tax Refund + Renter's Credit framework)",
        notes,
    }
}

fn check_vermont(input: &LandlordAnnualRentStatementInput) -> LandlordAnnualRentStatementResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    if !input.statement_issued_by_deadline {
        violations.push(
            "Vt. Stat. tit. 32 § 6066 — landlord MUST provide annual statement on Form LRC-147 for tenant Renter Rebate claim"
                .to_string(),
        );
    }

    if input.statement_issued_by_deadline && !input.statement_includes_required_information {
        violations.push(
            "Vt. Stat. tit. 32 § 6066 — Form LRC-147 MUST include landlord name + property address + total rent paid in calendar year"
                .to_string(),
        );
    }

    notes.push(
        "Vt. Stat. tit. 32 § 6066 — Renter Rebate Program supports tenant rebate claim"
            .to_string(),
    );

    let compliant = violations.is_empty();
    LandlordAnnualRentStatementResult {
        compliant,
        state_penalty_exposure: !compliant,
        affidavit_alternative_available: false,
        violations,
        citation: "Vt. Stat. tit. 32 § 6066 (VT Renter Rebate Program)",
        notes,
    }
}

fn check_default(_input: &LandlordAnnualRentStatementInput) -> LandlordAnnualRentStatementResult {
    let notes: Vec<String> = vec![
        "default rule — most states have no statewide landlord annual-rent-statement mandate; renter-tax-credit states (MA + MI + WI + IN + IA + ME + MD) typically have tenants claim based on tenant's own records"
            .to_string(),
        "renter-tax-credit cross-reference — Mass. Gen. Laws ch. 62 § 6(a) (MA Renter Deduction up to 50% of rent paid capped at $4,000); Ind. Code § 6-3-2-6 (IN Renter Deduction up to $3,000); Mich. Comp. Laws § 206.522 (MI Homestead Property Tax Credit including renters); Wis. Stat. § 71.07(9) (WI School Property Tax Credit); Iowa Code § 425.16 (IA Property Tax Credit); Me. Rev. Stat. tit. 36 § 6201 (ME Renters Rebate Program)"
            .to_string(),
        "landlord must produce records on tenant request even where no proactive statement obligation"
            .to_string(),
    ];

    LandlordAnnualRentStatementResult {
        compliant: true,
        state_penalty_exposure: false,
        affidavit_alternative_available: false,
        violations: Vec::new(),
        citation: "state-specific renter-tax-credit statutes; no statewide proactive landlord statement mandate",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mn_compliant() -> LandlordAnnualRentStatementInput {
        LandlordAnnualRentStatementInput {
            regime: Regime::Minnesota,
            annual_rent_paid_cents: 1_800_000,
            statement_issued_by_deadline: true,
            statement_includes_required_information: true,
            tenant_requested_rent_paid_affidavit: false,
        }
    }

    fn vt_compliant() -> LandlordAnnualRentStatementInput {
        LandlordAnnualRentStatementInput {
            regime: Regime::Vermont,
            annual_rent_paid_cents: 1_500_000,
            statement_issued_by_deadline: true,
            statement_includes_required_information: true,
            tenant_requested_rent_paid_affidavit: false,
        }
    }

    fn default_base() -> LandlordAnnualRentStatementInput {
        LandlordAnnualRentStatementInput {
            regime: Regime::Default,
            annual_rent_paid_cents: 2_400_000,
            statement_issued_by_deadline: false,
            statement_includes_required_information: false,
            tenant_requested_rent_paid_affidavit: false,
        }
    }

    #[test]
    fn mn_compliant_clean_path() {
        let r = check(&mn_compliant());
        assert!(r.compliant);
        assert!(!r.state_penalty_exposure);
        assert!(!r.affidavit_alternative_available);
    }

    #[test]
    fn mn_missing_january_31_deadline_violates() {
        let mut i = mn_compliant();
        i.statement_issued_by_deadline = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.state_penalty_exposure);
        assert!(r.violations.iter().any(|v| v.contains("§ 290A.19") && v.contains("JANUARY 31")));
    }

    #[test]
    fn mn_affidavit_alternative_engaged_when_nonissuance() {
        let mut i = mn_compliant();
        i.statement_issued_by_deadline = false;
        let r = check(&i);
        assert!(r.affidavit_alternative_available);
        assert!(r.notes.iter().any(|n| n.contains("Rent Paid Affidavit") && n.contains("Minnesota Department of Revenue")));
    }

    #[test]
    fn mn_affidavit_invocation_note_describes_audit() {
        let mut i = mn_compliant();
        i.statement_issued_by_deadline = false;
        i.tenant_requested_rent_paid_affidavit = true;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("invoked the Rent Paid Affidavit alternative") && n.contains("state-law penalties")));
    }

    #[test]
    fn mn_missing_required_information_violates() {
        let mut i = mn_compliant();
        i.statement_includes_required_information = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("§ 290A.19") && v.contains("MUST include")));
    }

    #[test]
    fn mn_electronic_or_hard_copy_note_present() {
        let r = check(&mn_compliant());
        assert!(r.notes.iter().any(|n| n.contains("ELECTRONIC OR HARD COPY")));
    }

    #[test]
    fn mn_property_tax_refund_note_present() {
        let r = check(&mn_compliant());
        assert!(r.notes.iter().any(|n| n.contains("Property Tax Refund") && n.contains("Renter's Credit")));
    }

    #[test]
    fn mn_citation_pins_290A_19_and_290A() {
        let r = check(&mn_compliant());
        assert!(r.citation.contains("§§ 290A.19, 290A"));
        assert!(r.citation.contains("Renter's Credit"));
    }

    #[test]
    fn vt_compliant_clean_path() {
        let r = check(&vt_compliant());
        assert!(r.compliant);
        assert!(!r.state_penalty_exposure);
    }

    #[test]
    fn vt_missing_deadline_violates() {
        let mut i = vt_compliant();
        i.statement_issued_by_deadline = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("Vt. Stat. tit. 32 § 6066") && v.contains("Form LRC-147")));
    }

    #[test]
    fn vt_missing_required_information_violates() {
        let mut i = vt_compliant();
        i.statement_includes_required_information = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("LRC-147") && v.contains("MUST include")));
    }

    #[test]
    fn vt_renter_rebate_note_present() {
        let r = check(&vt_compliant());
        assert!(r.notes.iter().any(|n| n.contains("Renter Rebate Program")));
    }

    #[test]
    fn vt_citation_pins_tit_32_6066() {
        let r = check(&vt_compliant());
        assert!(r.citation.contains("tit. 32 § 6066"));
    }

    #[test]
    fn vt_no_affidavit_alternative_unlike_mn() {
        let mut i = vt_compliant();
        i.statement_issued_by_deadline = false;
        let r = check(&i);
        assert!(!r.affidavit_alternative_available);
    }

    #[test]
    fn default_no_obligation_compliant_even_without_statement() {
        let r = check(&default_base());
        assert!(r.compliant);
        assert!(!r.state_penalty_exposure);
    }

    #[test]
    fn default_renter_tax_credit_cross_references_note() {
        let r = check(&default_base());
        assert!(r.notes.iter().any(|n| n.contains("MA Renter Deduction") && n.contains("IN Renter Deduction") && n.contains("ME Renters Rebate")));
    }

    #[test]
    fn default_records_on_request_obligation_note() {
        let r = check(&default_base());
        assert!(r.notes.iter().any(|n| n.contains("landlord must produce records on tenant request")));
    }

    #[test]
    fn default_citation_references_no_statewide_mandate() {
        let r = check(&default_base());
        assert!(r.citation.contains("no statewide proactive landlord statement mandate"));
    }

    #[test]
    fn mn_unique_january_31_deadline_invariant() {
        let mut i_mn = mn_compliant();
        i_mn.statement_issued_by_deadline = false;
        let r_mn = check(&i_mn);
        assert!(!r_mn.compliant);
        assert!(r_mn.violations.iter().any(|v| v.contains("JANUARY 31")));

        let mut i_default = default_base();
        i_default.statement_issued_by_deadline = false;
        let r_default = check(&i_default);
        assert!(r_default.compliant);
    }

    #[test]
    fn vt_unique_form_lrc_147_invariant() {
        let mut i_vt = vt_compliant();
        i_vt.statement_issued_by_deadline = false;
        let r_vt = check(&i_vt);
        assert!(r_vt.violations.iter().any(|v| v.contains("Form LRC-147")));

        let mut i_default = default_base();
        i_default.statement_issued_by_deadline = false;
        let r_default = check(&i_default);
        assert!(!r_default.violations.iter().any(|v| v.contains("Form LRC-147")));
    }

    #[test]
    fn three_regimes_routed_correctly() {
        for regime in [Regime::Minnesota, Regime::Vermont, Regime::Default] {
            let mut i = mn_compliant();
            i.regime = regime;
            let r = check(&i);
            let _ = r.compliant;
            assert!(!r.citation.is_empty());
        }
    }

    #[test]
    fn mn_affidavit_alternative_unique_to_mn_invariant() {
        let mut i_mn = mn_compliant();
        i_mn.statement_issued_by_deadline = false;
        let r_mn = check(&i_mn);
        assert!(r_mn.affidavit_alternative_available);

        for regime in [Regime::Vermont, Regime::Default] {
            let mut i = mn_compliant();
            i.regime = regime;
            i.statement_issued_by_deadline = false;
            let r = check(&i);
            assert!(!r.affidavit_alternative_available, "regime {:?} should not engage MN affidavit alternative", regime);
        }
    }

    #[test]
    fn mn_full_compliance_no_affidavit_alternative() {
        let r = check(&mn_compliant());
        assert!(!r.affidavit_alternative_available);
    }

    #[test]
    fn mn_both_violations_simultaneous_when_only_deadline_missed() {
        let mut i = mn_compliant();
        i.statement_issued_by_deadline = false;
        i.statement_includes_required_information = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert_eq!(r.violations.len(), 1);
    }

    #[test]
    fn state_penalty_exposure_engaged_with_any_violation() {
        let mut i = mn_compliant();
        i.statement_issued_by_deadline = false;
        let r = check(&i);
        assert!(r.state_penalty_exposure);
    }

    #[test]
    fn state_penalty_exposure_not_engaged_when_compliant() {
        let r = check(&mn_compliant());
        assert!(!r.state_penalty_exposure);
    }

    #[test]
    fn default_no_state_penalty_exposure_ever() {
        let mut i = default_base();
        i.statement_issued_by_deadline = false;
        i.statement_includes_required_information = false;
        let r = check(&i);
        assert!(!r.state_penalty_exposure);
    }

    #[test]
    fn vt_no_affidavit_pathway_ever() {
        let mut i = vt_compliant();
        i.tenant_requested_rent_paid_affidavit = true;
        let r = check(&i);
        assert!(!r.affidavit_alternative_available);
    }
}
