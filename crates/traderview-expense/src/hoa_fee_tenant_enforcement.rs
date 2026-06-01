//! HOA fee enforcement against tenant in single-family / townhome
//! / condominium rentals — when can a homeowners association
//! enforce delinquent dues DIRECTLY against the tenant rather than
//! the owner-landlord? Distinct from `hoa_rental_restriction`
//! (HOA's restrictions ON renting). Critical trader-landlord
//! operational risk: trader-landlord defaults on HOA dues → HOA
//! may seize rent directly from tenant + evict tenant for
//! nonpayment.
//!
//! **Four regimes:**
//!
//! **Florida — Fla. Stat. § 720.3085 (single-family HOA) +
//! § 718.116(11) (condominium)**. Most aggressive direct-tenant
//! enforcement. When parcel owner is delinquent in paying ANY
//! monetary obligation due to the association, the association may
//! DEMAND that the tenant pay SUBSEQUENT RENTAL PAYMENTS directly
//! to the association until all delinquent amounts are paid. Notice
//! by hand delivery OR United States mail required. **Tenant
//! immune from landlord claim** for rent timely paid to
//! association. **Tenant rent credit** against landlord-owed rent
//! equal to amount paid to HOA. **HOA may evict tenant** for
//! failure to pay after demand.
//!
//! **Texas — Tex. Prop. Code § 209.0064 (Texas Residential
//! Property Owners Protection Act)**. HOA enforces ONLY against
//! OWNER (landlord), not tenant. If lease passes through HOA dues
//! to tenant, landlord may evict tenant for nonpayment per lease
//! terms — but HOA itself has no direct authority over tenant.
//! § 209.0064 third-party collection requires (i) written notice
//! by certified mail to OWNER listing delinquent amounts +
//! payment plans, (ii) 45-day cure period.
//!
//! **California — Davis-Stirling Common Interest Development Act
//! (Cal. Civ. Code §§ 5650, 5710, 5715)**. HOA enforces only
//! against OWNER. No statutory direct-tenant collection mechanism.
//! HOA may foreclose on owner's interest, NOT tenant.
//!
//! **Default — HOA enforces only against owner**. Most states
//! follow CA / TX pattern — HOA has lien against owner's property
//! but no direct tenant authority. Federal Fair Debt Collection
//! Practices Act applies if HOA hires third-party collector.
//!
//! Citations: Fla. Stat. § 720.3085(8)(a)-(c) (FL single-family
//! HOA direct-tenant collection + immunity + credit); Fla. Stat.
//! § 718.116(11) (FL condominium); Tex. Prop. Code § 209.0064
//! (TX third-party collection + 45-day notice); Cal. Civ. Code
//! §§ 5650, 5710, 5715 (CA Davis-Stirling Act); 15 U.S.C. § 1692
//! (federal FDCPA for third-party collectors).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    Florida,
    Texas,
    California,
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HoaFeeTenantEnforcementInput {
    pub regime: Regime,
    /// Whether the owner-landlord is delinquent on HOA / condo
    /// association dues.
    pub landlord_delinquent_on_hoa_dues: bool,
    /// Whether the HOA / association has formally demanded the
    /// tenant pay rent directly to the association (FL §
    /// 720.3085(8) trigger).
    pub hoa_demanded_tenant_pay_directly: bool,
    /// Whether the HOA delivered the demand notice via hand
    /// delivery OR United States mail (FL § 720.3085(8)
    /// procedural requirement).
    pub written_notice_via_hand_or_mail: bool,
    /// Whether the tenant has actually paid rent directly to the
    /// HOA following demand.
    pub tenant_paid_hoa_directly: bool,
    /// Whether the lease independently requires the tenant to
    /// pay HOA dues (TX pass-through pattern). If true, landlord
    /// may evict for nonpayment per lease, but HOA itself has no
    /// direct tenant authority outside FL.
    pub lease_passes_hoa_dues_to_tenant: bool,
    /// Whether TX § 209.0064 third-party collection 45-day notice
    /// was provided to the OWNER (not tenant).
    pub tx_owner_45_day_notice_provided: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct HoaFeeTenantEnforcementResult {
    /// Whether the HOA has statutory authority to enforce dues
    /// directly against the tenant (rather than only the owner).
    pub hoa_can_enforce_against_tenant: bool,
    /// Whether the tenant is immune from landlord's rent claim
    /// for amounts paid to the HOA (FL § 720.3085(8)(b)).
    pub tenant_immunity_from_landlord_claim: bool,
    /// Whether the tenant has a credit against landlord-owed rent
    /// equal to amount paid to HOA (FL § 720.3085(8)(c)).
    pub tenant_rent_credit_engaged: bool,
    /// Whether the HOA may evict the tenant for failure to pay
    /// after written demand (FL § 720.3085(8)(d)).
    pub hoa_eviction_authority_engaged: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &HoaFeeTenantEnforcementInput) -> HoaFeeTenantEnforcementResult {
    match input.regime {
        Regime::Florida => check_florida(input),
        Regime::Texas => check_texas(input),
        Regime::California => check_california(input),
        Regime::Default => check_default(input),
    }
}

fn check_florida(input: &HoaFeeTenantEnforcementInput) -> HoaFeeTenantEnforcementResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    let demand_triggered = input.landlord_delinquent_on_hoa_dues
        && input.hoa_demanded_tenant_pay_directly;

    let mut hoa_enforce = false;
    let mut immunity = false;
    let mut credit = false;
    let mut eviction = false;

    if demand_triggered {
        if !input.written_notice_via_hand_or_mail {
            violations.push(
                "Fla. Stat. § 720.3085(8) — HOA demand to tenant MUST be by hand delivery OR United States mail; verbal demand insufficient"
                    .to_string(),
            );
        } else {
            hoa_enforce = true;
            if input.tenant_paid_hoa_directly {
                immunity = true;
                credit = true;
            }
            eviction = true;
        }
    }

    notes.push(
        "Fla. Stat. § 720.3085(8)(a) — HOA may demand subsequent rental payments directly to association when parcel owner delinquent on ANY monetary obligation"
            .to_string(),
    );
    notes.push(
        "Fla. Stat. § 720.3085(8)(b) — tenant IMMUNE from landlord claim for rent timely paid to association after written demand"
            .to_string(),
    );
    notes.push(
        "Fla. Stat. § 720.3085(8)(c) — tenant gets RENT CREDIT against landlord-owed rent equal to amount paid to association; tenant liability capped at amount due to landlord"
            .to_string(),
    );
    notes.push(
        "Fla. Stat. § 720.3085(8)(d) — HOA may sue tenant for eviction if tenant fails to pay after written demand"
            .to_string(),
    );
    notes.push(
        "Fla. Stat. § 718.116(11) — parallel rule for CONDOMINIUM associations (Chapter 718) — same direct-tenant collection mechanism applies"
            .to_string(),
    );

    HoaFeeTenantEnforcementResult {
        hoa_can_enforce_against_tenant: hoa_enforce,
        tenant_immunity_from_landlord_claim: immunity,
        tenant_rent_credit_engaged: credit,
        hoa_eviction_authority_engaged: eviction,
        violations,
        citation: "Fla. Stat. §§ 720.3085(8)(a)-(d), 718.116(11)",
        notes,
    }
}

fn check_texas(input: &HoaFeeTenantEnforcementInput) -> HoaFeeTenantEnforcementResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    if input.landlord_delinquent_on_hoa_dues && !input.tx_owner_45_day_notice_provided {
        violations.push(
            "Tex. Prop. Code § 209.0064 — HOA debt-collection-fee charge requires written certified-mail notice to OWNER listing delinquent amounts + payment plans + 45-day cure period"
                .to_string(),
        );
    }

    notes.push(
        "Tex. Prop. Code § 209.0064 — HOA enforces ONLY against OWNER; no statutory direct-tenant collection mechanism in Texas"
            .to_string(),
    );

    if input.lease_passes_hoa_dues_to_tenant {
        notes.push(
            "lease passes HOA dues to tenant — landlord may evict for nonpayment per lease terms; HOA itself still has no direct tenant authority under TX law"
                .to_string(),
        );
    }

    HoaFeeTenantEnforcementResult {
        hoa_can_enforce_against_tenant: false,
        tenant_immunity_from_landlord_claim: false,
        tenant_rent_credit_engaged: false,
        hoa_eviction_authority_engaged: false,
        violations,
        citation: "Tex. Prop. Code § 209.0064",
        notes,
    }
}

fn check_california(input: &HoaFeeTenantEnforcementInput) -> HoaFeeTenantEnforcementResult {
    let mut notes: Vec<String> = Vec::new();

    notes.push(
        "Cal. Civ. Code §§ 5650, 5710, 5715 (Davis-Stirling Common Interest Development Act) — HOA enforces ONLY against OWNER; no statutory direct-tenant collection mechanism"
            .to_string(),
    );
    notes.push(
        "Cal. Civ. Code § 5710 — HOA may foreclose on owner's interest in unit, NOT tenant"
            .to_string(),
    );

    if input.lease_passes_hoa_dues_to_tenant {
        notes.push(
            "lease passes HOA dues to tenant — landlord may evict for nonpayment per lease terms; HOA itself still has no direct tenant authority under CA Davis-Stirling Act"
                .to_string(),
        );
    }

    HoaFeeTenantEnforcementResult {
        hoa_can_enforce_against_tenant: false,
        tenant_immunity_from_landlord_claim: false,
        tenant_rent_credit_engaged: false,
        hoa_eviction_authority_engaged: false,
        violations: Vec::new(),
        citation: "Cal. Civ. Code §§ 5650, 5710, 5715 (Davis-Stirling Act)",
        notes,
    }
}

fn check_default(input: &HoaFeeTenantEnforcementInput) -> HoaFeeTenantEnforcementResult {
    let mut notes: Vec<String> = Vec::new();

    notes.push(
        "default rule — most states follow CA / TX pattern; HOA has lien against owner's property but NO direct tenant authority; federal FDCPA (15 U.S.C. § 1692) applies if HOA hires third-party collector"
            .to_string(),
    );

    if input.lease_passes_hoa_dues_to_tenant {
        notes.push(
            "lease passes HOA dues to tenant — landlord may evict for nonpayment per lease terms; HOA itself still has no direct tenant authority under default state law"
                .to_string(),
        );
    }

    HoaFeeTenantEnforcementResult {
        hoa_can_enforce_against_tenant: false,
        tenant_immunity_from_landlord_claim: false,
        tenant_rent_credit_engaged: false,
        hoa_eviction_authority_engaged: false,
        violations: Vec::new(),
        citation: "state-specific HOA / common-interest statute + 15 U.S.C. § 1692 (FDCPA)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fl_base() -> HoaFeeTenantEnforcementInput {
        HoaFeeTenantEnforcementInput {
            regime: Regime::Florida,
            landlord_delinquent_on_hoa_dues: true,
            hoa_demanded_tenant_pay_directly: true,
            written_notice_via_hand_or_mail: true,
            tenant_paid_hoa_directly: true,
            lease_passes_hoa_dues_to_tenant: false,
            tx_owner_45_day_notice_provided: false,
        }
    }

    fn tx_base() -> HoaFeeTenantEnforcementInput {
        HoaFeeTenantEnforcementInput {
            regime: Regime::Texas,
            landlord_delinquent_on_hoa_dues: true,
            hoa_demanded_tenant_pay_directly: false,
            written_notice_via_hand_or_mail: false,
            tenant_paid_hoa_directly: false,
            lease_passes_hoa_dues_to_tenant: false,
            tx_owner_45_day_notice_provided: true,
        }
    }

    fn ca_base() -> HoaFeeTenantEnforcementInput {
        HoaFeeTenantEnforcementInput {
            regime: Regime::California,
            landlord_delinquent_on_hoa_dues: true,
            hoa_demanded_tenant_pay_directly: false,
            written_notice_via_hand_or_mail: false,
            tenant_paid_hoa_directly: false,
            lease_passes_hoa_dues_to_tenant: false,
            tx_owner_45_day_notice_provided: false,
        }
    }

    fn default_base() -> HoaFeeTenantEnforcementInput {
        HoaFeeTenantEnforcementInput {
            regime: Regime::Default,
            landlord_delinquent_on_hoa_dues: true,
            hoa_demanded_tenant_pay_directly: false,
            written_notice_via_hand_or_mail: false,
            tenant_paid_hoa_directly: false,
            lease_passes_hoa_dues_to_tenant: false,
            tx_owner_45_day_notice_provided: false,
        }
    }

    #[test]
    fn fl_full_demand_with_notice_engages_enforcement() {
        let r = check(&fl_base());
        assert!(r.hoa_can_enforce_against_tenant);
        assert!(r.tenant_immunity_from_landlord_claim);
        assert!(r.tenant_rent_credit_engaged);
        assert!(r.hoa_eviction_authority_engaged);
    }

    #[test]
    fn fl_demand_without_written_notice_voids_enforcement() {
        let mut i = fl_base();
        i.written_notice_via_hand_or_mail = false;
        let r = check(&i);
        assert!(!r.hoa_can_enforce_against_tenant);
        assert!(!r.tenant_immunity_from_landlord_claim);
        assert!(r.violations.iter().any(|v| v.contains("§ 720.3085(8)") && v.contains("hand delivery OR United States mail")));
    }

    #[test]
    fn fl_no_landlord_delinquency_no_enforcement() {
        let mut i = fl_base();
        i.landlord_delinquent_on_hoa_dues = false;
        let r = check(&i);
        assert!(!r.hoa_can_enforce_against_tenant);
        assert!(!r.tenant_immunity_from_landlord_claim);
    }

    #[test]
    fn fl_no_demand_no_enforcement() {
        let mut i = fl_base();
        i.hoa_demanded_tenant_pay_directly = false;
        let r = check(&i);
        assert!(!r.hoa_can_enforce_against_tenant);
    }

    #[test]
    fn fl_demand_with_notice_but_tenant_not_paid_eviction_engaged() {
        let mut i = fl_base();
        i.tenant_paid_hoa_directly = false;
        let r = check(&i);
        assert!(r.hoa_can_enforce_against_tenant);
        assert!(r.hoa_eviction_authority_engaged);
        assert!(!r.tenant_immunity_from_landlord_claim);
        assert!(!r.tenant_rent_credit_engaged);
    }

    #[test]
    fn fl_condominium_parallel_rule_note_present() {
        let r = check(&fl_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 718.116(11)") && n.contains("CONDOMINIUM")));
    }

    #[test]
    fn fl_citation_pins_subsections_a_b_c_d_and_condo_section() {
        let r = check(&fl_base());
        assert!(r.citation.contains("§§ 720.3085(8)(a)-(d), 718.116(11)"));
    }

    #[test]
    fn fl_tenant_credit_note_describes_liability_cap() {
        let r = check(&fl_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 720.3085(8)(c)") && n.contains("liability capped")));
    }

    #[test]
    fn tx_owner_only_enforcement_invariant() {
        let r = check(&tx_base());
        assert!(!r.hoa_can_enforce_against_tenant);
        assert!(!r.tenant_immunity_from_landlord_claim);
        assert!(!r.hoa_eviction_authority_engaged);
    }

    #[test]
    fn tx_delinquent_without_45_day_notice_violation() {
        let mut i = tx_base();
        i.tx_owner_45_day_notice_provided = false;
        let r = check(&i);
        assert!(r.violations.iter().any(|v| v.contains("§ 209.0064") && v.contains("45-day cure period")));
    }

    #[test]
    fn tx_lease_passthrough_note_describes_landlord_eviction() {
        let mut i = tx_base();
        i.lease_passes_hoa_dues_to_tenant = true;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("lease passes HOA dues to tenant") && n.contains("landlord may evict")));
    }

    #[test]
    fn tx_citation_pins_209_0064() {
        let r = check(&tx_base());
        assert!(r.citation.contains("§ 209.0064"));
    }

    #[test]
    fn ca_owner_only_enforcement_invariant() {
        let r = check(&ca_base());
        assert!(!r.hoa_can_enforce_against_tenant);
        assert!(!r.tenant_immunity_from_landlord_claim);
    }

    #[test]
    fn ca_davis_stirling_foreclose_owner_not_tenant_note() {
        let r = check(&ca_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 5710") && n.contains("foreclose on owner's interest") && n.contains("NOT tenant")));
    }

    #[test]
    fn ca_citation_pins_davis_stirling_sections() {
        let r = check(&ca_base());
        assert!(r.citation.contains("§§ 5650, 5710, 5715"));
        assert!(r.citation.contains("Davis-Stirling Act"));
    }

    #[test]
    fn default_no_direct_tenant_authority_invariant() {
        let r = check(&default_base());
        assert!(!r.hoa_can_enforce_against_tenant);
        assert!(!r.tenant_immunity_from_landlord_claim);
        assert!(!r.tenant_rent_credit_engaged);
        assert!(!r.hoa_eviction_authority_engaged);
    }

    #[test]
    fn default_fdcpa_note_for_third_party_collector() {
        let r = check(&default_base());
        assert!(r.notes.iter().any(|n| n.contains("FDCPA") && n.contains("15 U.S.C. § 1692")));
    }

    #[test]
    fn florida_unique_direct_tenant_enforcement_invariant() {
        let r_fl = check(&fl_base());
        assert!(r_fl.hoa_can_enforce_against_tenant);

        for regime in [Regime::Texas, Regime::California, Regime::Default] {
            let mut i = fl_base();
            i.regime = regime;
            i.tx_owner_45_day_notice_provided = true;
            let r = check(&i);
            assert!(
                !r.hoa_can_enforce_against_tenant,
                "regime {:?} should not engage direct tenant enforcement",
                regime
            );
        }
    }

    #[test]
    fn florida_unique_tenant_immunity_invariant() {
        let r_fl = check(&fl_base());
        assert!(r_fl.tenant_immunity_from_landlord_claim);

        for regime in [Regime::Texas, Regime::California, Regime::Default] {
            let mut i = fl_base();
            i.regime = regime;
            i.tx_owner_45_day_notice_provided = true;
            let r = check(&i);
            assert!(
                !r.tenant_immunity_from_landlord_claim,
                "regime {:?} should not engage tenant immunity",
                regime
            );
        }
    }

    #[test]
    fn florida_unique_rent_credit_invariant() {
        let r_fl = check(&fl_base());
        assert!(r_fl.tenant_rent_credit_engaged);

        for regime in [Regime::Texas, Regime::California, Regime::Default] {
            let mut i = fl_base();
            i.regime = regime;
            i.tx_owner_45_day_notice_provided = true;
            let r = check(&i);
            assert!(
                !r.tenant_rent_credit_engaged,
                "regime {:?} should not engage rent credit",
                regime
            );
        }
    }

    #[test]
    fn florida_unique_hoa_eviction_authority_invariant() {
        let r_fl = check(&fl_base());
        assert!(r_fl.hoa_eviction_authority_engaged);

        for regime in [Regime::Texas, Regime::California, Regime::Default] {
            let mut i = fl_base();
            i.regime = regime;
            i.tx_owner_45_day_notice_provided = true;
            let r = check(&i);
            assert!(
                !r.hoa_eviction_authority_engaged,
                "regime {:?} should not engage HOA eviction authority",
                regime
            );
        }
    }

    #[test]
    fn four_regimes_routed_correctly() {
        for regime in [Regime::Florida, Regime::Texas, Regime::California, Regime::Default] {
            let mut i = fl_base();
            i.regime = regime;
            i.tx_owner_45_day_notice_provided = true;
            let r = check(&i);
            let _ = r.hoa_can_enforce_against_tenant;
            assert!(!r.citation.is_empty());
        }
    }

    #[test]
    fn lease_passthrough_note_in_three_non_florida_regimes() {
        for regime in [Regime::Texas, Regime::California, Regime::Default] {
            let mut i = fl_base();
            i.regime = regime;
            i.lease_passes_hoa_dues_to_tenant = true;
            i.tx_owner_45_day_notice_provided = true;
            let r = check(&i);
            assert!(r.notes.iter().any(|n| n.contains("lease passes HOA dues")), "regime {:?} should describe lease passthrough", regime);
        }
    }

    #[test]
    fn fl_delinquency_without_demand_no_enforcement() {
        let mut i = fl_base();
        i.hoa_demanded_tenant_pay_directly = false;
        let r = check(&i);
        assert!(!r.hoa_can_enforce_against_tenant);
        assert!(!r.tenant_immunity_from_landlord_claim);
    }

    #[test]
    fn fl_demand_without_delinquency_no_enforcement() {
        let mut i = fl_base();
        i.landlord_delinquent_on_hoa_dues = false;
        let r = check(&i);
        assert!(!r.hoa_can_enforce_against_tenant);
    }

    #[test]
    fn fl_four_subsection_notes_all_present() {
        let r = check(&fl_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 720.3085(8)(a)")));
        assert!(r.notes.iter().any(|n| n.contains("§ 720.3085(8)(b)")));
        assert!(r.notes.iter().any(|n| n.contains("§ 720.3085(8)(c)")));
        assert!(r.notes.iter().any(|n| n.contains("§ 720.3085(8)(d)")));
    }

    #[test]
    fn tx_clean_compliance_no_violations() {
        let r = check(&tx_base());
        assert!(r.violations.is_empty());
    }

    #[test]
    fn fl_clean_compliance_no_violations() {
        let r = check(&fl_base());
        assert!(r.violations.is_empty());
    }
}
