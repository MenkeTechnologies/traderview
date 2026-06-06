//! Multi-jurisdictional tenant lease guarantor disclosure
//! and restriction framework. Trader-landlord critical
//! because (1) lease guarantors are widely used by
//! landlords to secure rent payments from high-risk
//! tenants, but state law increasingly restricts both
//! the amount a guarantor can be obligated to pay AND
//! the procedural protections required when soliciting
//! guarantors; (2) NY HSTPA 2019 + DHCR guidance limits
//! ALL deposits and guarantor security to ONE MONTH'S
//! RENT for rent-stabilized apartments; (3) Federal FCRA
//! requirements apply when landlord pulls guarantor's
//! consumer credit report; (4) common-law suretyship
//! principles (Restatement (Third) of Suretyship and
//! Guaranty) STRICTLY CONSTRUE guarantor liability +
//! EXTINGUISH guaranty on material modifications absent
//! consent.
//!
//! Companion to tenant_data_privacy (FCRA + screening),
//! rental_application_denial_disclosure (adverse-action
//! notice), tenant_late_fee_cap, tenant_rent_receipt_
//! requirement.
//!
//! **New York HSTPA 2019 + DHCR Operational Bulletin
//! 2020-1** — for rent-stabilized apartments:
//! 1. Landlord may NOT demand additional security or
//!    guaranty from tenant + guarantor + third party
//!    that EXCEEDS ONE MONTH'S RENT in aggregate;
//! 2. Landlord may NOT retroactively require guarantor
//!    after initial lease signing;
//! 3. Tenant blacklist prohibition: landlord may NOT use
//!    Housing Court litigation history as basis for
//!    denial; $500-$1,000 civil penalty per violation;
//! 4. UCS records sealed under HSTPA 2019.
//!
//! **NY General Obligations Law § 5-701(a)(1) Statute of
//! Frauds** — lease guaranty for a term LONGER THAN ONE
//! YEAR must be IN WRITING signed by the guarantor or
//! authorized agent. Oral guaranty for 1-year-or-less
//! lease enforceable but harder to prove.
//!
//! **California Civ. Code Suretyship Framework**:
//! 1. **§ 2787-2856** — Common-law suretyship principles
//!    + statutory codification.
//! 2. **§ 2819 — Material modification rule** — guarantor
//!    DISCHARGED if creditor (landlord) materially
//!    modifies obligation (lease term, rent amount, etc.)
//!    WITHOUT GUARANTOR'S CONSENT.
//! 3. **§ 1670.5 — Unconscionability** — court may refuse
//!    to enforce guaranty contract or any clause thereof
//!    that is unconscionable at time made.
//! 4. **§ 2799 — Continuing guaranty revocation** —
//!    continuing guaranty may be revoked by guarantor at
//!    any time as to future transactions; pre-revocation
//!    obligations remain.
//!
//! **New Jersey** — Truth-in-Lending Disclosure + Limit
//! Act (TILDLA) + N.J.S.A. 46:8-26:
//! 1. Landlord must provide guarantor with COPY OF LEASE
//!    + all material modifications;
//! 2. Guaranty must specify EXACT MONETARY LIMIT;
//! 3. NJ Consumer Fraud Act overlay for residential
//!    guaranties.
//!
//! **Federal Fair Credit Reporting Act (15 USC § 1681 et
//! seq.)** — when landlord requests guarantor's consumer
//! credit report:
//! 1. **§ 1681b — Permissible purpose** — landlord must
//!    have valid permissible purpose under FCRA;
//! 2. **§ 1681e — Accuracy requirements** — landlord as
//!    USER must follow reasonable procedures;
//! 3. **§ 1681m — Adverse action notice** — if landlord
//!    denies guaranty based in whole or in part on
//!    consumer credit report, must provide guarantor
//!    with ADVERSE ACTION NOTICE including:
//!    - Identity of credit reporting agency;
//!    - Statement of guarantor's right to free copy
//!      within 60 days;
//!    - Statement of guarantor's right to dispute
//!      inaccurate information.
//! 4. **§ 1681n and § 1681o** — willful violation gets
//!    actual plus punitive damages plus attorney fees;
//!    negligent violation gets actual damages plus
//!    attorney fees.
//!
//! **Default — Common-law guaranty principles
//! (Restatement (Third) of Suretyship and Guaranty,
//! 1996)**:
//! 1. Guaranty STRICTLY CONSTRUED against the creditor
//!    (landlord);
//! 2. Material modification of underlying obligation
//!    WITHOUT GUARANTOR'S CONSENT extinguishes the
//!    guaranty (§ 41);
//! 3. Novation of the underlying lease extinguishes
//!    guaranty (§ 39);
//! 4. Limited-amount guaranties enforced strictly within
//!    monetary cap.
//!
//! Citations: NY HSTPA of 2019 (Pub. L. 2019-39, Pub. L.
//! 2019-36); NY DHCR Operational Bulletin 2020-1; NY GOL
//! § 5-701(a)(1); NY GOL § 7-108; Cal. Civ. Code § 2787;
//! Cal. Civ. Code § 2819; Cal. Civ. Code § 1670.5; Cal.
//! Civ. Code § 2799; N.J.S.A. 46:8-26; NJ Consumer Fraud
//! Act, N.J.S.A. 56:8-1 et seq.; Federal Fair Credit
//! Reporting Act, 15 USC § 1681 et seq.; 15 USC § 1681b
//! (permissible purpose); 15 USC § 1681e (accuracy); 15
//! USC § 1681m (adverse action); 15 USC § 1681n (willful
//! violation); 15 USC § 1681o (negligent violation);
//! Restatement (Third) of Suretyship and Guaranty (1996).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    NewYork,
    California,
    NewJersey,
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TenantLeaseGuarantorDisclosureInput {
    pub jurisdiction: Jurisdiction,
    /// Whether unit is rent-stabilized (NY) or
    /// rent-controlled (other jurisdictions).
    pub rent_stabilized_or_controlled: bool,
    /// Monthly rent in cents.
    pub monthly_rent_cents: u64,
    /// Aggregate security + guaranty amount demanded from
    /// tenant + guarantor in cents (NY HSTPA one-month
    /// cap test).
    pub total_security_and_guaranty_cents: u64,
    /// Whether guarantor was added AFTER initial lease
    /// signing (NY HSTPA prohibition).
    pub retroactive_guarantor_required: bool,
    /// Lease term in months (NY GOL § 5-701 Statute of
    /// Frauds engagement at > 12 months).
    pub lease_term_months: u32,
    /// Whether guaranty is IN WRITING signed by
    /// guarantor.
    pub guaranty_in_writing: bool,
    /// Whether lease was MATERIALLY MODIFIED after
    /// guaranty signed (rent increase + lease extension)
    /// without guarantor's consent.
    pub material_modification_without_consent: bool,
    /// Whether copy of lease + material modifications
    /// provided to guarantor (NJ requirement).
    pub copy_of_lease_provided_to_guarantor: bool,
    /// Whether guaranty specifies exact monetary limit
    /// (NJ + general best practice).
    pub guaranty_specifies_monetary_limit: bool,
    /// Whether landlord pulled guarantor's consumer
    /// credit report (FCRA gate).
    pub credit_report_pulled: bool,
    /// Whether landlord denied tenancy or guarantor based
    /// on credit report (FCRA adverse action trigger).
    pub adverse_action_taken: bool,
    /// Whether FCRA adverse action notice provided to
    /// guarantor.
    pub fcra_adverse_action_notice_provided: bool,
    /// Whether landlord used Housing Court litigation
    /// history (NY HSTPA blacklist prohibition).
    pub used_housing_court_blacklist: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TenantLeaseGuarantorDisclosureResult {
    pub jurisdiction: Jurisdiction,
    pub one_month_aggregate_cap_compliant: bool,
    pub retroactive_guarantor_prohibited: bool,
    pub statute_of_frauds_satisfied: bool,
    pub material_modification_extinguishes_guaranty: bool,
    pub fcra_adverse_action_notice_required: bool,
    pub blacklist_violation: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &TenantLeaseGuarantorDisclosureInput) -> TenantLeaseGuarantorDisclosureResult {
    let mut failure_reasons: Vec<String> = Vec::new();

    let one_month_aggregate_cap_compliant = match input.jurisdiction {
        Jurisdiction::NewYork if input.rent_stabilized_or_controlled => {
            input.total_security_and_guaranty_cents <= input.monthly_rent_cents
        }
        _ => true,
    };

    if input.jurisdiction == Jurisdiction::NewYork
        && input.rent_stabilized_or_controlled
        && input.total_security_and_guaranty_cents > input.monthly_rent_cents
    {
        failure_reasons.push(format!(
            "NY HSTPA 2019 + GOL § 7-108 + DHCR Operational Bulletin 2020-1 — for rent-stabilized apartments, landlord may NOT demand aggregate security + guaranty from tenant + guarantor + third party that EXCEEDS ONE MONTH'S RENT ({} cents); demanded {} cents",
            input.monthly_rent_cents, input.total_security_and_guaranty_cents
        ));
    }

    let retroactive_guarantor_prohibited = input.jurisdiction == Jurisdiction::NewYork
        && input.rent_stabilized_or_controlled
        && input.retroactive_guarantor_required;
    if retroactive_guarantor_prohibited {
        failure_reasons.push(
            "NY HSTPA 2019 + DHCR Operational Bulletin 2020-1 — landlord may NOT retroactively require guarantor after initial lease signing for rent-stabilized tenant".to_string(),
        );
    }

    let blacklist_violation =
        input.jurisdiction == Jurisdiction::NewYork && input.used_housing_court_blacklist;
    if blacklist_violation {
        failure_reasons.push(
            "NY HSTPA 2019 — tenant blacklist prohibition: landlord may NOT use Housing Court litigation history as basis for denial; $500-$1,000 civil penalty per violation; UCS records sealed under HSTPA 2019".to_string(),
        );
    }

    let statute_of_frauds_required = input.lease_term_months > 12;
    let statute_of_frauds_satisfied = !statute_of_frauds_required || input.guaranty_in_writing;
    if statute_of_frauds_required && !input.guaranty_in_writing {
        failure_reasons.push(
            "NY GOL § 5-701(a)(1) Statute of Frauds — lease guaranty for a term LONGER THAN ONE YEAR must be IN WRITING signed by the guarantor or authorized agent; oral guaranty for > 12-month lease is unenforceable".to_string(),
        );
    }

    let material_modification_extinguishes_guaranty = match input.jurisdiction {
        Jurisdiction::California => input.material_modification_without_consent,
        Jurisdiction::Default | Jurisdiction::NewYork | Jurisdiction::NewJersey => {
            input.material_modification_without_consent
        }
    };
    if material_modification_extinguishes_guaranty {
        match input.jurisdiction {
            Jurisdiction::California => {
                failure_reasons.push(
                    "Cal. Civ. Code § 2819 — MATERIAL MODIFICATION rule — guarantor DISCHARGED if creditor (landlord) materially modifies obligation (lease term, rent amount, etc.) WITHOUT GUARANTOR'S CONSENT".to_string(),
                );
            }
            _ => {
                failure_reasons.push(
                    "Restatement (Third) of Suretyship and Guaranty § 41 (1996) — guaranty EXTINGUISHED when underlying obligation MATERIALLY MODIFIED without guarantor's consent; rent increases + lease extensions are typically material modifications".to_string(),
                );
            }
        }
    }

    if input.jurisdiction == Jurisdiction::NewJersey {
        if !input.copy_of_lease_provided_to_guarantor {
            failure_reasons.push(
                "N.J.S.A. 46:8-26 + NJ Consumer Fraud Act — landlord must provide guarantor with COPY OF LEASE + all material modifications".to_string(),
            );
        }
        if !input.guaranty_specifies_monetary_limit {
            failure_reasons.push(
                "N.J.S.A. 46:8-26 + NJ Consumer Fraud Act — guaranty must specify EXACT MONETARY LIMIT to be enforceable".to_string(),
            );
        }
    }

    let fcra_adverse_action_notice_required =
        input.credit_report_pulled && input.adverse_action_taken;
    if fcra_adverse_action_notice_required && !input.fcra_adverse_action_notice_provided {
        failure_reasons.push(
            "15 USC § 1681m + § 1681n + § 1681o — Federal FCRA ADVERSE ACTION NOTICE required when landlord denies guaranty based in whole or in part on consumer credit report; notice must include (1) identity of credit reporting agency; (2) statement of guarantor's right to free copy within 60 days; (3) statement of right to dispute inaccurate information; willful violation = actual + punitive damages + attorney fees; negligent violation = actual damages + attorney fees".to_string(),
        );
    }

    let notes: Vec<String> = vec![
        "NY HSTPA 2019 + DHCR Operational Bulletin 2020-1 — for rent-stabilized apartments: landlord may NOT demand aggregate security + guaranty from tenant + guarantor + third party that EXCEEDS ONE MONTH'S RENT; may NOT retroactively require guarantor after initial lease signing".to_string(),
        "NY HSTPA 2019 — TENANT BLACKLIST PROHIBITION: landlord may NOT use Housing Court litigation history as basis for denial; $500-$1,000 civil penalty per violation; UCS records sealed under HSTPA 2019".to_string(),
        "NY GOL § 5-701(a)(1) Statute of Frauds — lease guaranty for a term LONGER THAN ONE YEAR must be IN WRITING signed by the guarantor or authorized agent; oral guaranty for > 12-month lease is unenforceable".to_string(),
        "Cal. Civ. Code § 2787-2856 — Common-law suretyship principles + statutory codification".to_string(),
        "Cal. Civ. Code § 2819 MATERIAL MODIFICATION rule — guarantor DISCHARGED if creditor materially modifies obligation (lease term, rent amount, etc.) WITHOUT GUARANTOR'S CONSENT".to_string(),
        "Cal. Civ. Code § 1670.5 UNCONSCIONABILITY — court may refuse to enforce guaranty contract or any clause thereof that is unconscionable at time made".to_string(),
        "Cal. Civ. Code § 2799 CONTINUING GUARANTY REVOCATION — continuing guaranty may be revoked by guarantor at any time as to future transactions; pre-revocation obligations remain".to_string(),
        "N.J.S.A. 46:8-26 + NJ Consumer Fraud Act — landlord must provide guarantor with COPY OF LEASE + all material modifications; guaranty must specify EXACT MONETARY LIMIT".to_string(),
        "Federal Fair Credit Reporting Act (15 USC § 1681 et seq.) — § 1681b PERMISSIBLE PURPOSE required for landlord to obtain guarantor's credit report; § 1681e accuracy requirements; § 1681m ADVERSE ACTION NOTICE required with identity of credit reporting agency + 60-day free-copy right + dispute right".to_string(),
        "15 USC § 1681n + § 1681o — FCRA willful violation = ACTUAL + PUNITIVE damages + attorney fees; negligent violation = actual damages + attorney fees".to_string(),
        "Restatement (Third) of Suretyship and Guaranty (1996) — § 41 MATERIAL MODIFICATION EXTINGUISHES; § 39 NOVATION EXTINGUISHES; guaranty STRICTLY CONSTRUED against the creditor (landlord); limited-amount guaranties enforced strictly within monetary cap".to_string(),
        "Trader-landlord critical: (1) NY HSTPA aggregate one-month cap on security + guaranty for rent-stabilized; (2) FCRA adverse-action notice required when guarantor application denied based on credit report; (3) common-law material-modification rule means subsequent rent increases or lease extensions extinguish guaranty without guarantor consent; (4) NJ + best practice requires exact monetary cap on guaranty for enforceability".to_string(),
    ];

    TenantLeaseGuarantorDisclosureResult {
        jurisdiction: input.jurisdiction,
        one_month_aggregate_cap_compliant,
        retroactive_guarantor_prohibited,
        statute_of_frauds_satisfied,
        material_modification_extinguishes_guaranty,
        fcra_adverse_action_notice_required,
        blacklist_violation,
        failure_reasons,
        citation: "NY HSTPA of 2019; NY DHCR Operational Bulletin 2020-1; NY GOL § 5-701(a)(1); NY GOL § 7-108; Cal. Civ. Code § 2787; Cal. Civ. Code § 2819; Cal. Civ. Code § 1670.5; Cal. Civ. Code § 2799; N.J.S.A. 46:8-26; NJ Consumer Fraud Act, N.J.S.A. 56:8-1 et seq.; Federal Fair Credit Reporting Act, 15 USC § 1681 et seq.; 15 USC § 1681b; 15 USC § 1681e; 15 USC § 1681m; 15 USC § 1681n; 15 USC § 1681o; Restatement (Third) of Suretyship and Guaranty (1996)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ny_rent_stabilized_baseline() -> TenantLeaseGuarantorDisclosureInput {
        TenantLeaseGuarantorDisclosureInput {
            jurisdiction: Jurisdiction::NewYork,
            rent_stabilized_or_controlled: true,
            monthly_rent_cents: 200_000,
            total_security_and_guaranty_cents: 200_000,
            retroactive_guarantor_required: false,
            lease_term_months: 12,
            guaranty_in_writing: true,
            material_modification_without_consent: false,
            copy_of_lease_provided_to_guarantor: true,
            guaranty_specifies_monetary_limit: true,
            credit_report_pulled: false,
            adverse_action_taken: false,
            fcra_adverse_action_notice_provided: false,
            used_housing_court_blacklist: false,
        }
    }

    #[test]
    fn ny_one_month_aggregate_compliant() {
        let r = check(&ny_rent_stabilized_baseline());
        assert!(r.one_month_aggregate_cap_compliant);
    }

    #[test]
    fn ny_aggregate_exceeds_one_month_violation() {
        let mut i = ny_rent_stabilized_baseline();
        i.total_security_and_guaranty_cents = 300_000;
        let r = check(&i);
        assert!(!r.one_month_aggregate_cap_compliant);
        assert!(r.failure_reasons.iter().any(|f| f.contains("HSTPA 2019")
            && f.contains("§ 7-108")
            && f.contains("ONE MONTH'S RENT")));
    }

    #[test]
    fn ny_non_rent_stabilized_no_aggregate_cap() {
        let mut i = ny_rent_stabilized_baseline();
        i.rent_stabilized_or_controlled = false;
        i.total_security_and_guaranty_cents = 1_000_000;
        let r = check(&i);
        assert!(r.one_month_aggregate_cap_compliant);
    }

    #[test]
    fn ny_retroactive_guarantor_prohibited() {
        let mut i = ny_rent_stabilized_baseline();
        i.retroactive_guarantor_required = true;
        let r = check(&i);
        assert!(r.retroactive_guarantor_prohibited);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("HSTPA 2019") && f.contains("retroactively require guarantor")));
    }

    #[test]
    fn ny_blacklist_violation_500_to_1000_penalty() {
        let mut i = ny_rent_stabilized_baseline();
        i.used_housing_court_blacklist = true;
        let r = check(&i);
        assert!(r.blacklist_violation);
        assert!(r.failure_reasons.iter().any(|f| f.contains("HSTPA 2019")
            && f.contains("blacklist")
            && f.contains("$500-$1,000")));
    }

    #[test]
    fn statute_of_frauds_24_month_oral_unenforceable() {
        let mut i = ny_rent_stabilized_baseline();
        i.lease_term_months = 24;
        i.guaranty_in_writing = false;
        let r = check(&i);
        assert!(!r.statute_of_frauds_satisfied);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 5-701(a)(1)")
            && f.contains("Statute of Frauds")
            && f.contains("IN WRITING")));
    }

    #[test]
    fn statute_of_frauds_12_month_oral_enforceable() {
        let mut i = ny_rent_stabilized_baseline();
        i.lease_term_months = 12;
        i.guaranty_in_writing = false;
        let r = check(&i);
        assert!(r.statute_of_frauds_satisfied);
    }

    #[test]
    fn statute_of_frauds_12_month_boundary() {
        let mut i = ny_rent_stabilized_baseline();
        i.lease_term_months = 12;
        i.guaranty_in_writing = false;
        let r = check(&i);
        assert!(r.statute_of_frauds_satisfied);
    }

    #[test]
    fn statute_of_frauds_13_month_boundary_requires_writing() {
        let mut i = ny_rent_stabilized_baseline();
        i.lease_term_months = 13;
        i.guaranty_in_writing = false;
        let r = check(&i);
        assert!(!r.statute_of_frauds_satisfied);
    }

    #[test]
    fn ca_material_modification_extinguishes_2819() {
        let mut i = ny_rent_stabilized_baseline();
        i.jurisdiction = Jurisdiction::California;
        i.material_modification_without_consent = true;
        let r = check(&i);
        assert!(r.material_modification_extinguishes_guaranty);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 2819")
            && f.contains("MATERIAL MODIFICATION")
            && f.contains("DISCHARGED")));
    }

    #[test]
    fn default_material_modification_restatement_41() {
        let mut i = ny_rent_stabilized_baseline();
        i.jurisdiction = Jurisdiction::Default;
        i.material_modification_without_consent = true;
        let r = check(&i);
        assert!(r.material_modification_extinguishes_guaranty);
        assert!(r.failure_reasons.iter().any(|f| f
            .contains("Restatement (Third) of Suretyship and Guaranty § 41")
            && f.contains("EXTINGUISHED")));
    }

    #[test]
    fn nj_no_copy_of_lease_violation() {
        let mut i = ny_rent_stabilized_baseline();
        i.jurisdiction = Jurisdiction::NewJersey;
        i.copy_of_lease_provided_to_guarantor = false;
        let r = check(&i);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("46:8-26") && f.contains("COPY OF LEASE")));
    }

    #[test]
    fn nj_no_monetary_limit_violation() {
        let mut i = ny_rent_stabilized_baseline();
        i.jurisdiction = Jurisdiction::NewJersey;
        i.guaranty_specifies_monetary_limit = false;
        let r = check(&i);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("46:8-26") && f.contains("MONETARY LIMIT")));
    }

    #[test]
    fn fcra_adverse_action_notice_required_engages() {
        let mut i = ny_rent_stabilized_baseline();
        i.credit_report_pulled = true;
        i.adverse_action_taken = true;
        i.fcra_adverse_action_notice_provided = false;
        let r = check(&i);
        assert!(r.fcra_adverse_action_notice_required);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 1681m")
            && f.contains("ADVERSE ACTION NOTICE")
            && f.contains("60 days")));
    }

    #[test]
    fn fcra_no_adverse_action_no_notice_required() {
        let mut i = ny_rent_stabilized_baseline();
        i.credit_report_pulled = true;
        i.adverse_action_taken = false;
        let r = check(&i);
        assert!(!r.fcra_adverse_action_notice_required);
    }

    #[test]
    fn fcra_no_credit_report_no_notice_required() {
        let mut i = ny_rent_stabilized_baseline();
        i.credit_report_pulled = false;
        i.adverse_action_taken = true;
        let r = check(&i);
        assert!(!r.fcra_adverse_action_notice_required);
    }

    #[test]
    fn fcra_adverse_action_with_notice_no_violation() {
        let mut i = ny_rent_stabilized_baseline();
        i.credit_report_pulled = true;
        i.adverse_action_taken = true;
        i.fcra_adverse_action_notice_provided = true;
        let r = check(&i);
        assert!(!r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 1681m") && f.contains("ADVERSE ACTION NOTICE")));
    }

    #[test]
    fn jurisdiction_truth_table_four_cells() {
        for jur in [
            Jurisdiction::NewYork,
            Jurisdiction::California,
            Jurisdiction::NewJersey,
            Jurisdiction::Default,
        ] {
            let mut i = ny_rent_stabilized_baseline();
            i.jurisdiction = jur;
            let r = check(&i);
            assert_eq!(r.jurisdiction, jur);
        }
    }

    #[test]
    fn ny_uniquely_engages_blacklist_prohibition_invariant() {
        let mut ny = ny_rent_stabilized_baseline();
        ny.used_housing_court_blacklist = true;
        let r_ny = check(&ny);
        assert!(r_ny.blacklist_violation);

        for jur in [
            Jurisdiction::California,
            Jurisdiction::NewJersey,
            Jurisdiction::Default,
        ] {
            let mut i = ny_rent_stabilized_baseline();
            i.jurisdiction = jur;
            i.used_housing_court_blacklist = true;
            let r = check(&i);
            assert!(!r.blacklist_violation, "jur={:?}", jur);
        }
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&ny_rent_stabilized_baseline());
        assert!(r.citation.contains("NY HSTPA of 2019"));
        assert!(r.citation.contains("NY DHCR Operational Bulletin 2020-1"));
        assert!(r.citation.contains("NY GOL § 5-701(a)(1)"));
        assert!(r.citation.contains("NY GOL § 7-108"));
        assert!(r.citation.contains("Cal. Civ. Code § 2787"));
        assert!(r.citation.contains("Cal. Civ. Code § 2819"));
        assert!(r.citation.contains("Cal. Civ. Code § 1670.5"));
        assert!(r.citation.contains("Cal. Civ. Code § 2799"));
        assert!(r.citation.contains("N.J.S.A. 46:8-26"));
        assert!(r.citation.contains("NJ Consumer Fraud Act"));
        assert!(r.citation.contains("15 USC § 1681 et seq."));
        assert!(r.citation.contains("15 USC § 1681b"));
        assert!(r.citation.contains("15 USC § 1681e"));
        assert!(r.citation.contains("15 USC § 1681m"));
        assert!(r.citation.contains("15 USC § 1681n"));
        assert!(r.citation.contains("15 USC § 1681o"));
        assert!(r
            .citation
            .contains("Restatement (Third) of Suretyship and Guaranty (1996)"));
    }

    #[test]
    fn note_pins_ny_hstpa_one_month_aggregate() {
        let r = check(&ny_rent_stabilized_baseline());
        assert!(r.notes.iter().any(|n| n.contains("HSTPA 2019")
            && n.contains("Operational Bulletin 2020-1")
            && n.contains("ONE MONTH'S RENT")
            && n.contains("retroactively require guarantor")));
    }

    #[test]
    fn note_pins_ny_blacklist_prohibition() {
        let r = check(&ny_rent_stabilized_baseline());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("TENANT BLACKLIST PROHIBITION")
                && n.contains("$500-$1,000")
                && n.contains("UCS records sealed")));
    }

    #[test]
    fn note_pins_ny_gol_5_701_statute_of_frauds() {
        let r = check(&ny_rent_stabilized_baseline());
        assert!(r.notes.iter().any(|n| n.contains("§ 5-701(a)(1)")
            && n.contains("Statute of Frauds")
            && n.contains("LONGER THAN ONE YEAR")
            && n.contains("IN WRITING")));
    }

    #[test]
    fn note_pins_ca_2819_material_modification() {
        let r = check(&ny_rent_stabilized_baseline());
        assert!(r.notes.iter().any(|n| n.contains("§ 2819")
            && n.contains("MATERIAL MODIFICATION")
            && n.contains("DISCHARGED")
            && n.contains("WITHOUT GUARANTOR'S CONSENT")));
    }

    #[test]
    fn note_pins_ca_1670_5_unconscionability() {
        let r = check(&ny_rent_stabilized_baseline());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1670.5") && n.contains("UNCONSCIONABILITY")));
    }

    #[test]
    fn note_pins_ca_2799_continuing_guaranty_revocation() {
        let r = check(&ny_rent_stabilized_baseline());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 2799") && n.contains("CONTINUING GUARANTY REVOCATION")));
    }

    #[test]
    fn note_pins_nj_46_8_26_lease_copy_monetary_limit() {
        let r = check(&ny_rent_stabilized_baseline());
        assert!(r.notes.iter().any(|n| n.contains("N.J.S.A. 46:8-26")
            && n.contains("COPY OF LEASE")
            && n.contains("EXACT MONETARY LIMIT")));
    }

    #[test]
    fn note_pins_fcra_adverse_action_notice() {
        let r = check(&ny_rent_stabilized_baseline());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1681b PERMISSIBLE PURPOSE")
                && n.contains("§ 1681m ADVERSE ACTION NOTICE")
                && n.contains("60-day free-copy right")
                && n.contains("dispute right")));
    }

    #[test]
    fn note_pins_fcra_willful_punitive_damages() {
        let r = check(&ny_rent_stabilized_baseline());
        assert!(r.notes.iter().any(|n| n.contains("§ 1681n")
            && n.contains("§ 1681o")
            && n.contains("willful")
            && n.contains("PUNITIVE damages")));
    }

    #[test]
    fn note_pins_restatement_third_suretyship_1996() {
        let r = check(&ny_rent_stabilized_baseline());
        assert!(r.notes.iter().any(|n| n
            .contains("Restatement (Third) of Suretyship and Guaranty (1996)")
            && n.contains("§ 41 MATERIAL MODIFICATION EXTINGUISHES")
            && n.contains("§ 39 NOVATION EXTINGUISHES")
            && n.contains("STRICTLY CONSTRUED")));
    }

    #[test]
    fn note_pins_trader_landlord_critical_summary() {
        let r = check(&ny_rent_stabilized_baseline());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Trader-landlord critical")
                && n.contains("NY HSTPA aggregate one-month cap")
                && n.contains("FCRA adverse-action notice")
                && n.contains("material-modification rule")
                && n.contains("exact monetary cap")));
    }

    #[test]
    fn multiple_failures_stack() {
        let mut i = ny_rent_stabilized_baseline();
        i.total_security_and_guaranty_cents = 300_000;
        i.retroactive_guarantor_required = true;
        i.used_housing_court_blacklist = true;
        i.lease_term_months = 24;
        i.guaranty_in_writing = false;
        i.material_modification_without_consent = true;
        i.credit_report_pulled = true;
        i.adverse_action_taken = true;
        i.fcra_adverse_action_notice_provided = false;
        let r = check(&i);
        assert!(r.failure_reasons.len() >= 5);
    }
}
