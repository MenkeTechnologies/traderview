//! Tenant estoppel certificate requirements and
//! protections — trader-landlord critical because
//! traders who own rental property routinely need
//! signed estoppel certificates from tenants when
//! REFINANCING the property OR SELLING the property to
//! a buyer. The estoppel certifies (1) the actual lease
//! terms (rent, security deposit, term, modifications),
//! (2) the absence of landlord defaults, and (3) that
//! the lease is in full force and effect. A buyer or
//! lender RELIES on the estoppel; the tenant is
//! BOUND BY ITS CONTENTS under promissory estoppel.
//!
//! Most commercial form leases REQUIRE the tenant to
//! sign an estoppel within a SHORT WINDOW (10-15
//! business days standard) and contain DEEMED-ADMISSION
//! CLAUSES (failure to return = deemed true) plus
//! ATTORNEY-IN-FACT CLAUSES (landlord may sign on
//! tenant's behalf).
//!
//! Companion to lease_disclosures, lease_copy_delivery,
//! tenant_lease_guarantor_disclosure (iter 433),
//! tenant_rights_statement_disclosure, lease_waiver_
//! enforceability, landlord_identification_disclosure.
//!
//! **Three-jurisdiction framework**:
//!
//! - **New York** — both commercial and residential
//!   use estoppels but ONLY enforceable when lease
//!   contains EXPRESS WRITTEN PROVISION requiring
//!   tenant cooperation; NY GOL § 5-703 statute of
//!   frauds covers leases > 1 year (estoppel terms in
//!   such leases must be in writing); NY case law:
//!   common-law promissory estoppel binds tenant on
//!   reliance by buyer/lender; deemed-admission
//!   clauses generally enforceable in COMMERCIAL
//!   leases but VOID AS UNCONSCIONABLE in residential
//!   under NY GOL § 5-321 / RPL § 235-c.
//!
//! - **California** — Cal. Civ. Code § 1962
//!   (landlord identification disclosure) applies
//!   indirectly; commercial leases routinely require
//!   estoppel under express lease provision; CA tenant
//!   bound by materially false statements relied on
//!   by third party (promissory estoppel doctrine);
//!   Cal. Civ. Code § 1668 (no exculpation for fraud)
//!   limits scope of deemed-admission clauses.
//!
//! - **Default / Restatement** — Restatement (Second)
//!   of Contracts § 90 promissory estoppel binds
//!   tenant on (1) clear and definite promise/
//!   statement, (2) foreseeable reliance, (3) actual
//!   reliance, (4) injustice avoidable only by
//!   enforcement; Uniform Commercial Code does NOT
//!   apply to real estate leases.
//!
//! **Required estoppel certificate contents** (industry
//! standard + ALTA/ACSM survey practice):
//!
//! 1. Lease commencement date + expiration date;
//! 2. Current monthly rent amount + paid-through date;
//! 3. Security deposit amount + form (cash / letter of
//!    credit);
//! 4. All modifications and amendments listed by date;
//! 5. No landlord default / known breach;
//! 6. No prepaid rent beyond current month;
//! 7. No assignments or sublets without consent;
//! 8. Lease unmodified and in full force and effect;
//! 9. Tenant's current address for notices.
//!
//! **Response window** — standard COMMERCIAL form
//! requires 10-15 BUSINESS DAYS; failure to return
//! within window triggers:
//! - DEEMED-ADMISSION (everything in landlord's
//!   proposed estoppel deemed TRUE);
//! - ATTORNEY-IN-FACT (landlord may execute on
//!   tenant's behalf);
//! - MONETARY PENALTY (some leases impose $50-$500
//!   per day);
//! - EVENT OF DEFAULT (some leases treat refusal as
//!   material breach + eviction trigger).
//!
//! **Tenant protections against false estoppels**:
//! - CANNOT bind tenant to FACTS NOT KNOWN to tenant
//!   (Cal. Civ. Code § 1668; NY common law);
//! - CANNOT waive STATUTORY rights (e.g., rent-
//!   stabilization protections in NY; security
//!   deposit limits in CA);
//! - CANNOT serve as PRE-DISPUTE WAIVER of yet-to-
//!   accrue claims;
//! - DEEMED-ADMISSION clauses VOID in NY RESIDENTIAL
//!   leases as unconscionable under RPL § 235-c.
//!
//! **Trader-landlord critical fact patterns**:
//! 1. Trader refinancing $5M apartment building —
//!    lender requires estoppel from every tenant;
//!    tenant who refuses delays closing and triggers
//!    rate-lock expiry damages.
//! 2. Trader selling rental — buyer's lender requires
//!    estoppels; failure to obtain triggers contract
//!    contingency.
//! 3. Tenant signs estoppel reciting $1,200 rent when
//!    actual rent was $1,000 — buyer/lender relies;
//!    tenant later cannot claim lower rent as it is
//!    bound by representation.
//! 4. NY rent-stabilized tenant signs estoppel waiving
//!    succession rights — VOID under DHCR rules
//!    (cannot waive statutory protections).
//! 5. Residential tenant signs estoppel with material
//!    misrepresentation by landlord (e.g., reciting
//!    "no defaults" when habitability breach exists)
//!    — tenant retains right to assert breach despite
//!    estoppel.
//!
//! Citations: Restatement (Second) of Contracts § 90;
//! NY GOL § 5-703 (Statute of Frauds — leases);
//! NY GOL § 5-321 (residential lease unconscionability);
//! NY RPL § 235-c (residential lease unconscionability);
//! Cal. Civ. Code § 1962 (landlord identification);
//! Cal. Civ. Code § 1668 (no exculpation for fraud);
//! ALTA/ACSM estoppel form standards; Fannie Mae
//! Multifamily Guide Form 6402 Tenant Estoppel
//! Certificate.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    NewYork,
    California,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LeaseUseType {
    Commercial,
    Residential,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TenantEstoppelCertificateInput {
    pub jurisdiction: Jurisdiction,
    pub lease_use_type: LeaseUseType,
    /// Whether lease contains express written estoppel
    /// cooperation provision.
    pub lease_has_express_estoppel_provision: bool,
    /// Whether tenant responded within the demanded
    /// response window.
    pub tenant_responded_within_window: bool,
    /// Demanded response window in business days.
    pub response_window_business_days: u32,
    /// Whether landlord's proposed estoppel form includes
    /// deemed-admission clause for non-response.
    pub form_includes_deemed_admission_clause: bool,
    /// Whether landlord's form includes attorney-in-fact
    /// clause empowering landlord to sign on tenant's
    /// behalf.
    pub form_includes_attorney_in_fact_clause: bool,
    /// Whether estoppel form attempts to waive statutory
    /// rent-control / rent-stabilization protections (NY).
    pub form_waives_statutory_protections: bool,
    /// Whether estoppel form attempts to serve as pre-
    /// dispute waiver of yet-to-accrue claims.
    pub form_serves_as_pre_dispute_waiver: bool,
    /// Whether tenant signed an estoppel containing a
    /// material misrepresentation.
    pub tenant_signed_with_material_misrep: bool,
    /// Whether the material misrepresentation was
    /// induced by landlord's own false statement.
    pub misrep_induced_by_landlord: bool,
    /// Whether buyer / lender actually relied on the
    /// estoppel in closing the refinance / sale.
    pub third_party_relied_on_estoppel: bool,
    /// Whether the estoppel contents are within tenant's
    /// PERSONAL KNOWLEDGE (factual matters tenant can
    /// verify).
    pub contents_within_tenant_knowledge: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TenantEstoppelCertificateResult {
    pub estoppel_demand_enforceable: bool,
    pub deemed_admission_clause_enforceable: bool,
    pub attorney_in_fact_clause_enforceable: bool,
    pub statutory_waiver_attempt_void: bool,
    pub pre_dispute_waiver_attempt_void: bool,
    pub tenant_bound_by_signed_estoppel: bool,
    pub tenant_retains_rights_despite_estoppel: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(
    input: &TenantEstoppelCertificateInput,
) -> TenantEstoppelCertificateResult {
    let mut failure_reasons: Vec<String> = Vec::new();

    let estoppel_demand_enforceable = input.lease_has_express_estoppel_provision;

    let deemed_admission_clause_enforceable = match (
        input.jurisdiction,
        input.lease_use_type,
    ) {
        (Jurisdiction::NewYork, LeaseUseType::Residential) => false,
        _ => input.form_includes_deemed_admission_clause
            && input.lease_has_express_estoppel_provision,
    };

    let attorney_in_fact_clause_enforceable = match (
        input.jurisdiction,
        input.lease_use_type,
    ) {
        (Jurisdiction::NewYork, LeaseUseType::Residential) => false,
        _ => input.form_includes_attorney_in_fact_clause
            && input.lease_has_express_estoppel_provision,
    };

    let statutory_waiver_attempt_void = input.form_waives_statutory_protections;
    let pre_dispute_waiver_attempt_void = input.form_serves_as_pre_dispute_waiver;

    let tenant_bound_by_signed_estoppel = input.tenant_signed_with_material_misrep
        && input.third_party_relied_on_estoppel
        && input.contents_within_tenant_knowledge
        && !input.misrep_induced_by_landlord;

    let tenant_retains_rights_despite_estoppel = (input.tenant_signed_with_material_misrep
        && input.misrep_induced_by_landlord)
        || !input.contents_within_tenant_knowledge
        || input.form_waives_statutory_protections
        || input.form_serves_as_pre_dispute_waiver;

    if !estoppel_demand_enforceable {
        failure_reasons.push(
            "Lease lacks EXPRESS WRITTEN ESTOPPEL COOPERATION PROVISION; tenant has NO duty to sign requested estoppel; landlord cannot compel signature absent lease provision (Restatement (Second) of Contracts § 90 promissory estoppel does not impose freestanding duty to sign)".to_string(),
        );
    }

    if matches!(
        (input.jurisdiction, input.lease_use_type),
        (Jurisdiction::NewYork, LeaseUseType::Residential)
    ) {
        if input.form_includes_deemed_admission_clause {
            failure_reasons.push(
                "NY RPL § 235-c — deemed-admission estoppel clause UNCONSCIONABLE and VOID in residential leases; clause cannot bind tenant via non-response".to_string(),
            );
        }
        if input.form_includes_attorney_in_fact_clause {
            failure_reasons.push(
                "NY RPL § 235-c — attorney-in-fact provision UNCONSCIONABLE and VOID in residential leases; landlord cannot execute estoppel on tenant's behalf".to_string(),
            );
        }
    }

    if input.form_waives_statutory_protections {
        failure_reasons.push(
            "Estoppel form attempts to waive STATUTORY PROTECTIONS (rent-stabilization succession rights, security deposit limits, habitability warranty); VOID as against public policy; tenant retains statutory rights regardless of signed estoppel".to_string(),
        );
    }

    if input.form_serves_as_pre_dispute_waiver {
        failure_reasons.push(
            "Estoppel form attempts to serve as PRE-DISPUTE WAIVER of yet-to-accrue claims; VOID as against public policy; cannot waive claims not yet matured".to_string(),
        );
    }

    if input.tenant_signed_with_material_misrep && input.misrep_induced_by_landlord {
        failure_reasons.push(
            "Material misrepresentation INDUCED BY LANDLORD; tenant NOT BOUND by estoppel; Cal. Civ. Code § 1668 (no exculpation for fraud) plus common-law fraud-in-the-inducement defense; tenant retains underlying claim".to_string(),
        );
    }

    if input.tenant_signed_with_material_misrep
        && !input.contents_within_tenant_knowledge
    {
        failure_reasons.push(
            "Estoppel contents NOT WITHIN TENANT'S PERSONAL KNOWLEDGE; cannot be bound to facts tenant could not have known; estoppel binding limited to matters within tenant's knowledge at signing".to_string(),
        );
    }

    if !input.tenant_responded_within_window
        && input.lease_has_express_estoppel_provision
        && input.form_includes_deemed_admission_clause
        && !matches!(
            (input.jurisdiction, input.lease_use_type),
            (Jurisdiction::NewYork, LeaseUseType::Residential)
        )
    {
        failure_reasons.push(format!(
            "Tenant FAILED to respond within {} business days; deemed-admission clause TRIGGERED; landlord's proposed estoppel contents DEEMED TRUE; tenant bound by representations in form even without signature",
            input.response_window_business_days
        ));
    }

    if tenant_bound_by_signed_estoppel {
        failure_reasons.push(
            "Tenant SIGNED estoppel with material misrepresentation; third party (buyer/lender) RELIED on representation; tenant BOUND under promissory estoppel doctrine (Restatement (Second) of Contracts § 90); cannot later contradict signed statement to detriment of relying party".to_string(),
        );
    }

    let notes: Vec<String> = vec![
        "Estoppel certificate certifies (1) actual lease terms (rent + security deposit + term + modifications), (2) absence of landlord defaults, (3) lease in full force and effect; buyer/lender RELIES on representations during refinance or sale; tenant BOUND BY CONTENTS under promissory estoppel".to_string(),
        "Restatement (Second) of Contracts § 90 — PROMISSORY ESTOPPEL binds tenant on (1) clear and definite promise/statement, (2) foreseeable reliance, (3) actual reliance by buyer/lender, (4) injustice avoidable only by enforcement; Uniform Commercial Code does NOT apply to real estate leases".to_string(),
        "Standard COMMERCIAL response window: 10-15 BUSINESS DAYS; failure to return triggers DEEMED-ADMISSION (contents deemed true) + ATTORNEY-IN-FACT (landlord signs on tenant's behalf) + MONETARY PENALTY ($50-$500/day) + EVENT OF DEFAULT (some leases treat as material breach + eviction trigger)".to_string(),
        "NY GOL § 5-321 — landlord cannot exempt itself from liability for negligence under residential lease provision; residential lease deemed-admission and attorney-in-fact clauses VOID AS UNCONSCIONABLE under NY RPL § 235-c".to_string(),
        "NY RPL § 235-c — residential lease unconscionability statute; courts may refuse to enforce ANY clause found unconscionable at time made; deemed-admission and attorney-in-fact clauses in residential context routinely struck".to_string(),
        "NY GOL § 5-703 — Statute of Frauds for leases > 1 year requires written instrument signed by party to be charged; estoppel certificate confirming lease terms generally satisfies writing requirement for the underlying lease terms".to_string(),
        "Cal. Civ. Code § 1962 — landlord identification disclosure (owner name + agent for service of process) required on every residential lease; related to but distinct from estoppel certificate practice".to_string(),
        "Cal. Civ. Code § 1668 — NO EXCULPATION FOR FRAUD; estoppel form cannot bind tenant where material misrepresentation was INDUCED BY LANDLORD; fraud-in-the-inducement defense available regardless of signature".to_string(),
        "Required estoppel contents (industry standard + ALTA/ACSM survey practice + Fannie Mae Multifamily Guide Form 6402): (1) lease commencement + expiration dates; (2) current monthly rent + paid-through date; (3) security deposit amount + form; (4) all modifications/amendments listed; (5) no landlord default / known breach; (6) no prepaid rent beyond current month; (7) no assignments / sublets without consent; (8) lease unmodified and in full force and effect; (9) tenant's current address for notices".to_string(),
        "Tenant protections against false estoppels: (1) cannot bind tenant to FACTS NOT KNOWN to tenant; (2) cannot waive STATUTORY rights (rent-stabilization succession in NY; security deposit limits in CA); (3) cannot serve as PRE-DISPUTE WAIVER of yet-to-accrue claims; (4) deemed-admission clauses VOID in NY residential as unconscionable under RPL § 235-c".to_string(),
        "Trader-landlord critical fact patterns: (1) trader refinancing $5M apartment building — lender requires estoppel from every tenant; tenant refusal delays closing + triggers rate-lock expiry damages; (2) trader selling rental — buyer's lender requires estoppels; failure to obtain triggers contract contingency; (3) tenant signs estoppel reciting $1,200 rent when actual rent $1,000 — tenant BOUND by representation; (4) NY rent-stabilized tenant signs estoppel waiving succession rights — VOID under DHCR rules; (5) residential tenant signs estoppel with landlord-induced misrep on no-defaults — tenant retains breach claim".to_string(),
        "Three-jurisdiction framework: NEW YORK (commercial enforceable with express provision; residential deemed-admission VOID under RPL § 235-c); CALIFORNIA (commercial enforceable with express provision; Cal. Civ. Code § 1668 limits fraud exculpation); DEFAULT/RESTATEMENT (promissory estoppel § 90; reliance + injustice test)".to_string(),
        "Companion to lease_disclosures + lease_copy_delivery + tenant_lease_guarantor_disclosure + tenant_rights_statement_disclosure + lease_waiver_enforceability + landlord_identification_disclosure".to_string(),
    ];

    TenantEstoppelCertificateResult {
        estoppel_demand_enforceable,
        deemed_admission_clause_enforceable,
        attorney_in_fact_clause_enforceable,
        statutory_waiver_attempt_void,
        pre_dispute_waiver_attempt_void,
        tenant_bound_by_signed_estoppel,
        tenant_retains_rights_despite_estoppel,
        failure_reasons,
        citation: "Restatement (Second) of Contracts § 90; NY GOL § 5-703 (Statute of Frauds — leases); NY GOL § 5-321 (residential lease unconscionability); NY RPL § 235-c (residential lease unconscionability); Cal. Civ. Code § 1962 (landlord identification); Cal. Civ. Code § 1668 (no exculpation for fraud); ALTA/ACSM estoppel form standards; Fannie Mae Multifamily Guide Form 6402 Tenant Estoppel Certificate",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn commercial_with_provision() -> TenantEstoppelCertificateInput {
        TenantEstoppelCertificateInput {
            jurisdiction: Jurisdiction::Default,
            lease_use_type: LeaseUseType::Commercial,
            lease_has_express_estoppel_provision: true,
            tenant_responded_within_window: true,
            response_window_business_days: 10,
            form_includes_deemed_admission_clause: true,
            form_includes_attorney_in_fact_clause: true,
            form_waives_statutory_protections: false,
            form_serves_as_pre_dispute_waiver: false,
            tenant_signed_with_material_misrep: false,
            misrep_induced_by_landlord: false,
            third_party_relied_on_estoppel: true,
            contents_within_tenant_knowledge: true,
        }
    }

    #[test]
    fn commercial_with_express_provision_enforceable() {
        let r = check(&commercial_with_provision());
        assert!(r.estoppel_demand_enforceable);
        assert!(r.deemed_admission_clause_enforceable);
        assert!(r.attorney_in_fact_clause_enforceable);
    }

    #[test]
    fn lease_without_express_provision_not_enforceable() {
        let mut i = commercial_with_provision();
        i.lease_has_express_estoppel_provision = false;
        let r = check(&i);
        assert!(!r.estoppel_demand_enforceable);
        assert!(!r.deemed_admission_clause_enforceable);
        assert!(!r.attorney_in_fact_clause_enforceable);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("EXPRESS WRITTEN ESTOPPEL COOPERATION PROVISION")));
    }

    #[test]
    fn ny_residential_deemed_admission_void() {
        let mut i = commercial_with_provision();
        i.jurisdiction = Jurisdiction::NewYork;
        i.lease_use_type = LeaseUseType::Residential;
        let r = check(&i);
        assert!(!r.deemed_admission_clause_enforceable);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("NY RPL § 235-c")
            && f.contains("deemed-admission")
            && f.contains("UNCONSCIONABLE")));
    }

    #[test]
    fn ny_residential_attorney_in_fact_void() {
        let mut i = commercial_with_provision();
        i.jurisdiction = Jurisdiction::NewYork;
        i.lease_use_type = LeaseUseType::Residential;
        let r = check(&i);
        assert!(!r.attorney_in_fact_clause_enforceable);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("NY RPL § 235-c")
            && f.contains("attorney-in-fact")
            && f.contains("UNCONSCIONABLE")));
    }

    #[test]
    fn ny_commercial_deemed_admission_enforceable() {
        let mut i = commercial_with_provision();
        i.jurisdiction = Jurisdiction::NewYork;
        i.lease_use_type = LeaseUseType::Commercial;
        let r = check(&i);
        assert!(r.deemed_admission_clause_enforceable);
    }

    #[test]
    fn california_commercial_deemed_admission_enforceable() {
        let mut i = commercial_with_provision();
        i.jurisdiction = Jurisdiction::California;
        i.lease_use_type = LeaseUseType::Commercial;
        let r = check(&i);
        assert!(r.deemed_admission_clause_enforceable);
        assert!(r.attorney_in_fact_clause_enforceable);
    }

    #[test]
    fn statutory_waiver_attempt_void() {
        let mut i = commercial_with_provision();
        i.form_waives_statutory_protections = true;
        let r = check(&i);
        assert!(r.statutory_waiver_attempt_void);
        assert!(r.tenant_retains_rights_despite_estoppel);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("STATUTORY PROTECTIONS")
            && f.contains("VOID")));
    }

    #[test]
    fn pre_dispute_waiver_attempt_void() {
        let mut i = commercial_with_provision();
        i.form_serves_as_pre_dispute_waiver = true;
        let r = check(&i);
        assert!(r.pre_dispute_waiver_attempt_void);
        assert!(r.tenant_retains_rights_despite_estoppel);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("PRE-DISPUTE WAIVER")
            && f.contains("VOID")));
    }

    #[test]
    fn tenant_bound_by_signed_estoppel_promissory_estoppel() {
        let mut i = commercial_with_provision();
        i.tenant_signed_with_material_misrep = true;
        i.third_party_relied_on_estoppel = true;
        i.contents_within_tenant_knowledge = true;
        i.misrep_induced_by_landlord = false;
        let r = check(&i);
        assert!(r.tenant_bound_by_signed_estoppel);
        assert!(!r.tenant_retains_rights_despite_estoppel);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("SIGNED estoppel")
            && f.contains("Restatement (Second) of Contracts § 90")));
    }

    #[test]
    fn landlord_induced_misrep_voids_binding_effect() {
        let mut i = commercial_with_provision();
        i.tenant_signed_with_material_misrep = true;
        i.misrep_induced_by_landlord = true;
        let r = check(&i);
        assert!(!r.tenant_bound_by_signed_estoppel);
        assert!(r.tenant_retains_rights_despite_estoppel);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("INDUCED BY LANDLORD")
            && f.contains("Cal. Civ. Code § 1668")));
    }

    #[test]
    fn contents_outside_tenant_knowledge_voids_binding() {
        let mut i = commercial_with_provision();
        i.tenant_signed_with_material_misrep = true;
        i.contents_within_tenant_knowledge = false;
        let r = check(&i);
        assert!(!r.tenant_bound_by_signed_estoppel);
        assert!(r.tenant_retains_rights_despite_estoppel);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("NOT WITHIN TENANT'S PERSONAL KNOWLEDGE")));
    }

    #[test]
    fn failure_to_respond_within_window_triggers_deemed_admission() {
        let mut i = commercial_with_provision();
        i.tenant_responded_within_window = false;
        i.response_window_business_days = 10;
        let r = check(&i);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("FAILED to respond within 10 business days")
            && f.contains("deemed-admission")
            && f.contains("DEEMED TRUE")));
    }

    #[test]
    fn ny_residential_no_response_no_deemed_admission_trigger() {
        let mut i = commercial_with_provision();
        i.jurisdiction = Jurisdiction::NewYork;
        i.lease_use_type = LeaseUseType::Residential;
        i.tenant_responded_within_window = false;
        let r = check(&i);
        assert!(!r.failure_reasons.iter().any(|f|
            f.contains("deemed-admission clause TRIGGERED")));
    }

    #[test]
    fn no_third_party_reliance_no_binding_effect() {
        let mut i = commercial_with_provision();
        i.tenant_signed_with_material_misrep = true;
        i.third_party_relied_on_estoppel = false;
        i.contents_within_tenant_knowledge = true;
        i.misrep_induced_by_landlord = false;
        let r = check(&i);
        assert!(!r.tenant_bound_by_signed_estoppel);
    }

    #[test]
    fn fifteen_day_window_response_failure() {
        let mut i = commercial_with_provision();
        i.tenant_responded_within_window = false;
        i.response_window_business_days = 15;
        let r = check(&i);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("FAILED to respond within 15 business days")));
    }

    #[test]
    fn jurisdiction_truth_table_six_cells() {
        let cases = [
            (Jurisdiction::NewYork, LeaseUseType::Commercial, true),
            (Jurisdiction::NewYork, LeaseUseType::Residential, false),
            (Jurisdiction::California, LeaseUseType::Commercial, true),
            (Jurisdiction::California, LeaseUseType::Residential, true),
            (Jurisdiction::Default, LeaseUseType::Commercial, true),
            (Jurisdiction::Default, LeaseUseType::Residential, true),
        ];
        for (j, use_type, exp) in cases {
            let mut i = commercial_with_provision();
            i.jurisdiction = j;
            i.lease_use_type = use_type;
            let r = check(&i);
            assert_eq!(
                r.deemed_admission_clause_enforceable, exp,
                "j={:?} use={:?}", j, use_type
            );
        }
    }

    #[test]
    fn ny_residential_uniquely_voids_deemed_admission_invariant() {
        let mut ny_res = commercial_with_provision();
        ny_res.jurisdiction = Jurisdiction::NewYork;
        ny_res.lease_use_type = LeaseUseType::Residential;
        let r_ny_res = check(&ny_res);
        assert!(!r_ny_res.deemed_admission_clause_enforceable);

        let mut ny_com = commercial_with_provision();
        ny_com.jurisdiction = Jurisdiction::NewYork;
        ny_com.lease_use_type = LeaseUseType::Commercial;
        let r_ny_com = check(&ny_com);
        assert!(r_ny_com.deemed_admission_clause_enforceable);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&commercial_with_provision());
        assert!(r.citation.contains("Restatement (Second) of Contracts § 90"));
        assert!(r.citation.contains("NY GOL § 5-703"));
        assert!(r.citation.contains("NY GOL § 5-321"));
        assert!(r.citation.contains("NY RPL § 235-c"));
        assert!(r.citation.contains("Cal. Civ. Code § 1962"));
        assert!(r.citation.contains("Cal. Civ. Code § 1668"));
        assert!(r.citation.contains("ALTA/ACSM"));
        assert!(r.citation.contains("Fannie Mae Multifamily Guide Form 6402"));
    }

    #[test]
    fn note_pins_estoppel_function() {
        let r = check(&commercial_with_provision());
        assert!(r.notes.iter().any(|n|
            n.contains("certifies")
            && n.contains("RELIES")
            && n.contains("BOUND BY CONTENTS")));
    }

    #[test]
    fn note_pins_restatement_section_90_promissory_estoppel() {
        let r = check(&commercial_with_provision());
        assert!(r.notes.iter().any(|n|
            n.contains("Restatement (Second) of Contracts § 90")
            && n.contains("PROMISSORY ESTOPPEL")
            && n.contains("clear and definite promise")
            && n.contains("foreseeable reliance")));
    }

    #[test]
    fn note_pins_response_window_consequences() {
        let r = check(&commercial_with_provision());
        assert!(r.notes.iter().any(|n|
            n.contains("10-15 BUSINESS DAYS")
            && n.contains("DEEMED-ADMISSION")
            && n.contains("ATTORNEY-IN-FACT")
            && n.contains("MONETARY PENALTY")
            && n.contains("EVENT OF DEFAULT")));
    }

    #[test]
    fn note_pins_ny_gol_5_321() {
        let r = check(&commercial_with_provision());
        assert!(r.notes.iter().any(|n|
            n.contains("NY GOL § 5-321")
            && n.contains("residential lease")));
    }

    #[test]
    fn note_pins_ny_rpl_235_c() {
        let r = check(&commercial_with_provision());
        assert!(r.notes.iter().any(|n|
            n.contains("NY RPL § 235-c")
            && n.contains("unconscionability")));
    }

    #[test]
    fn note_pins_ny_gol_5_703_statute_of_frauds() {
        let r = check(&commercial_with_provision());
        assert!(r.notes.iter().any(|n|
            n.contains("NY GOL § 5-703")
            && n.contains("Statute of Frauds")
            && n.contains("> 1 year")));
    }

    #[test]
    fn note_pins_ca_civ_code_1962() {
        let r = check(&commercial_with_provision());
        assert!(r.notes.iter().any(|n|
            n.contains("Cal. Civ. Code § 1962")
            && n.contains("landlord identification")));
    }

    #[test]
    fn note_pins_ca_civ_code_1668_no_fraud_exculpation() {
        let r = check(&commercial_with_provision());
        assert!(r.notes.iter().any(|n|
            n.contains("Cal. Civ. Code § 1668")
            && n.contains("NO EXCULPATION FOR FRAUD")
            && n.contains("INDUCED BY LANDLORD")));
    }

    #[test]
    fn note_pins_required_contents_nine_items() {
        let r = check(&commercial_with_provision());
        assert!(r.notes.iter().any(|n|
            n.contains("Required estoppel contents")
            && n.contains("ALTA/ACSM")
            && n.contains("Fannie Mae Multifamily Guide Form 6402")
            && n.contains("(1) lease commencement")
            && n.contains("(9) tenant's current address")));
    }

    #[test]
    fn note_pins_tenant_protections_four() {
        let r = check(&commercial_with_provision());
        assert!(r.notes.iter().any(|n|
            n.contains("Tenant protections against false estoppels")
            && n.contains("FACTS NOT KNOWN")
            && n.contains("STATUTORY rights")
            && n.contains("PRE-DISPUTE WAIVER")
            && n.contains("VOID in NY residential")));
    }

    #[test]
    fn note_pins_trader_fact_patterns_five() {
        let r = check(&commercial_with_provision());
        assert!(r.notes.iter().any(|n|
            n.contains("Trader-landlord critical fact patterns")
            && n.contains("$5M apartment building")
            && n.contains("rate-lock expiry")
            && n.contains("DHCR rules")));
    }

    #[test]
    fn note_pins_three_jurisdiction_framework() {
        let r = check(&commercial_with_provision());
        assert!(r.notes.iter().any(|n|
            n.contains("Three-jurisdiction framework")
            && n.contains("NEW YORK")
            && n.contains("CALIFORNIA")
            && n.contains("DEFAULT/RESTATEMENT")));
    }

    #[test]
    fn note_pins_companion_modules() {
        let r = check(&commercial_with_provision());
        assert!(r.notes.iter().any(|n|
            n.contains("Companion to lease_disclosures")
            && n.contains("tenant_lease_guarantor_disclosure")
            && n.contains("landlord_identification_disclosure")));
    }

    #[test]
    fn multiple_failures_stack() {
        let mut i = commercial_with_provision();
        i.lease_has_express_estoppel_provision = false;
        i.form_waives_statutory_protections = true;
        i.form_serves_as_pre_dispute_waiver = true;
        let r = check(&i);
        assert!(r.failure_reasons.len() >= 3);
    }

    #[test]
    fn tenant_retains_rights_pre_dispute_waiver_invariant() {
        let mut i = commercial_with_provision();
        i.form_serves_as_pre_dispute_waiver = true;
        let r = check(&i);
        assert!(r.tenant_retains_rights_despite_estoppel);
    }

    #[test]
    fn tenant_retains_rights_statutory_waiver_invariant() {
        let mut i = commercial_with_provision();
        i.form_waives_statutory_protections = true;
        let r = check(&i);
        assert!(r.tenant_retains_rights_despite_estoppel);
    }

    #[test]
    fn deemed_admission_requires_express_provision_invariant() {
        let mut no_provision = commercial_with_provision();
        no_provision.lease_has_express_estoppel_provision = false;
        let r_no = check(&no_provision);
        assert!(!r_no.deemed_admission_clause_enforceable);

        let with_provision = commercial_with_provision();
        let r_with = check(&with_provision);
        assert!(r_with.deemed_admission_clause_enforceable);
    }

    #[test]
    fn ny_residential_attorney_in_fact_uniquely_void_invariant() {
        let mut ny_res = commercial_with_provision();
        ny_res.jurisdiction = Jurisdiction::NewYork;
        ny_res.lease_use_type = LeaseUseType::Residential;
        let r_ny_res = check(&ny_res);
        assert!(!r_ny_res.attorney_in_fact_clause_enforceable);

        let mut ca_res = commercial_with_provision();
        ca_res.jurisdiction = Jurisdiction::California;
        ca_res.lease_use_type = LeaseUseType::Residential;
        let r_ca_res = check(&ca_res);
        assert!(r_ca_res.attorney_in_fact_clause_enforceable);
    }
}
