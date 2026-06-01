//! Mid-tenancy temporary relocation rights — when a landlord
//! requires the tenant to temporarily vacate the unit for
//! substantial repairs / renovation / abatement work, with the
//! tenant retaining the right to return after work is complete.
//!
//! Distinct from `tenant_relocation_assistance` (no-fault permanent
//! eviction relocation dollars), `lease_termination_catastrophic_
//! damage` (full lease termination after fire/flood), and
//! `demolition_tenant_notice` (notice of permanent unit demolition).
//! This module addresses ONLY the TEMPORARY VACATE-AND-RETURN
//! pathway during landlord-initiated capital improvements or
//! substantial remodel work.
//!
//! Three regimes:
//!
//! **California — Cal. Civ. Code § 1946.2(d)(2) ("substantial
//! remodel" just-cause) + § 1942.5(b) (temporary relocation)**.
//! "Substantial remodel" defined as replacement or substantial
//! modification of structural / electrical / plumbing / mechanical
//! system that REQUIRES the tenant to vacate the residential real
//! property for AT LEAST 30 DAYS. Cosmetic work (painting,
//! decorating, minor repairs) does NOT qualify. Tenant retains
//! right to return at completion under § 1946.2(d) just-cause
//! framework. Landlord must provide written notice and may need to
//! pay relocation assistance equal to one month's rent under
//! § 1946.2(d)(3) if treated as no-fault termination.
//!
//! **New Jersey — N.J.S.A. 2A:18-61.1(g) (Anti-Eviction Act
//! renovation removal)**. Permits termination when landlord seeks
//! to permanently retire the premises from residential use OR
//! perform substantial conversion / rehabilitation requiring tenant
//! to vacate. § 2A:18-61.11 requires landlord to OFFER TEMPORARY
//! ALTERNATIVE HOUSING or PAY for tenant's relocation expenses.
//!
//! **Washington — RCW 59.18.085 (Tenant relocation assistance for
//! displacement)**. Limited tenant relocation assistance when
//! landlord requires tenant to vacate for repair / maintenance /
//! abatement. Local jurisdictions (Seattle SMC 22.210, Bellingham
//! Tenant Protections Ordinance) impose additional requirements.
//!
//! **Default — lease + common-law habitability**. Most states lack
//! a statewide temporary-relocation statute; lease terms and
//! common-law habitability doctrine control. Some municipalities
//! (San Francisco Rent Ordinance § 37.9(a)(11), Long Beach
//! Substantial Remodel-Related Tenant Displacement Ordinance)
//! impose local requirements.
//!
//! Citations: Cal. Civ. Code § 1946.2(d)(2) (substantial remodel
//! definition); § 1946.2(d)(3) (one-month relocation assistance);
//! § 1942.5(b) (temporary relocation framework); N.J.S.A.
//! 2A:18-61.1(g) (NJ renovation removal); § 2A:18-61.11 (NJ
//! alternative housing or relocation expenses); RCW 59.18.085 (WA
//! tenant relocation assistance); SF Rent Ordinance § 37.9(a)(11)
//! (SF temporary eviction for capital improvements); Long Beach
//! SRTD Ordinance.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    California,
    NewJersey,
    Washington,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkType {
    /// Substantial structural / electrical / plumbing / mechanical
    /// system replacement OR hazardous material abatement (lead /
    /// mold / asbestos) requiring vacancy ≥ 30 days.
    SubstantialRemodel,
    /// Cosmetic improvements — painting, decorating, minor repairs.
    /// Does NOT qualify as substantial remodel; tenant retains
    /// occupancy.
    CosmeticImprovement,
    /// Capital improvement (major repair) requiring temporary
    /// vacancy but not rising to substantial remodel.
    CapitalImprovement,
    /// Emergency repair (fire / flood damage, structural failure).
    /// May entail different procedural treatment.
    EmergencyRepair,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TemporaryRelocationInput {
    pub regime: Regime,
    pub work_type: WorkType,
    /// Whether the work requires vacancy ≥ 30 days (Cal. Civ. Code
    /// § 1946.2(d)(2) threshold).
    pub vacancy_required_30_plus_days: bool,
    /// Whether the landlord provided written notice in advance.
    pub written_notice_provided: bool,
    /// Whether the landlord offered comparable alternative housing
    /// during the work.
    pub alternative_housing_offered: bool,
    /// Whether the landlord paid relocation expenses / fees (CA
    /// one-month rent; NJ alternative housing or relocation
    /// expenses; WA limited).
    pub relocation_assistance_paid: bool,
    /// Whether the tenant retains right to return at completion of
    /// work.
    pub tenant_right_to_return_preserved: bool,
    /// Local jurisdiction has stricter rules (SF Rent Ordinance,
    /// Long Beach SRTD, Seattle SMC 22.210, etc.).
    pub local_jurisdiction_overlay_applies: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TemporaryRelocationResult {
    pub compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &TemporaryRelocationInput) -> TemporaryRelocationResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    if matches!(input.work_type, WorkType::CosmeticImprovement) {
        notes.push(
            "cosmetic improvements (painting, decorating, minor repairs) do NOT qualify as substantial remodel under Cal. Civ. Code § 1946.2(d)(2); tenant retains occupancy without relocation requirement"
                .to_string(),
        );
        return TemporaryRelocationResult {
            compliant: true,
            violations,
            citation: citation_for(input.regime),
            notes,
        };
    }

    match input.regime {
        Regime::California => check_california(input, &mut violations, &mut notes),
        Regime::NewJersey => check_new_jersey(input, &mut violations, &mut notes),
        Regime::Washington => check_washington(input, &mut violations, &mut notes),
        Regime::Default => check_default(input, &mut violations, &mut notes),
    }
}

fn check_california(
    input: &TemporaryRelocationInput,
    violations: &mut Vec<String>,
    notes: &mut Vec<String>,
) -> TemporaryRelocationResult {
    if matches!(input.work_type, WorkType::SubstantialRemodel) && !input.vacancy_required_30_plus_days {
        violations.push(
            "Cal. Civ. Code § 1946.2(d)(2) — substantial remodel requires vacancy ≥ 30 days; work not meeting 30-day threshold does NOT qualify as substantial remodel just-cause"
                .to_string(),
        );
    }

    if !input.written_notice_provided {
        violations.push(
            "Cal. Civ. Code § 1946.2 — written notice required for substantial remodel termination / temporary relocation"
                .to_string(),
        );
    }

    if !input.relocation_assistance_paid && !input.alternative_housing_offered {
        violations.push(
            "Cal. Civ. Code § 1946.2(d)(3) — landlord must pay one-month rent relocation assistance OR provide alternative housing"
                .to_string(),
        );
    }

    if !input.tenant_right_to_return_preserved {
        violations.push(
            "Cal. Civ. Code § 1942.5(b) — tenant retains right to return at completion of temporary work; permanent displacement without proper § 1946.2 just-cause termination is unlawful"
                .to_string(),
        );
    }

    if input.local_jurisdiction_overlay_applies {
        notes.push(
            "local jurisdiction overlay — SF Rent Ordinance § 37.9(a)(11), Long Beach SRTD, or other municipal ordinance may impose additional requirements"
                .to_string(),
        );
    }

    TemporaryRelocationResult {
        compliant: violations.is_empty(),
        violations: violations.clone(),
        citation: citation_for(Regime::California),
        notes: notes.clone(),
    }
}

fn check_new_jersey(
    input: &TemporaryRelocationInput,
    violations: &mut Vec<String>,
    notes: &mut Vec<String>,
) -> TemporaryRelocationResult {
    if !input.written_notice_provided {
        violations.push(
            "N.J.S.A. 2A:18-61.1(g) — written notice required for Anti-Eviction Act renovation removal"
                .to_string(),
        );
    }

    if !input.alternative_housing_offered && !input.relocation_assistance_paid {
        violations.push(
            "N.J.S.A. 2A:18-61.11 — landlord must OFFER temporary alternative housing OR PAY tenant relocation expenses"
                .to_string(),
        );
    }

    if !input.tenant_right_to_return_preserved {
        notes.push(
            "N.J.S.A. 2A:18-61.1(g) permits termination for substantial conversion or rehabilitation; tenant return right depends on whether work is rehabilitation or permanent conversion"
                .to_string(),
        );
    }

    TemporaryRelocationResult {
        compliant: violations.is_empty(),
        violations: violations.clone(),
        citation: citation_for(Regime::NewJersey),
        notes: notes.clone(),
    }
}

fn check_washington(
    input: &TemporaryRelocationInput,
    violations: &mut Vec<String>,
    notes: &mut Vec<String>,
) -> TemporaryRelocationResult {
    if !input.written_notice_provided {
        violations.push(
            "RCW 59.18.085 — written notice required for tenant relocation due to displacement"
                .to_string(),
        );
    }

    if !input.relocation_assistance_paid {
        violations.push(
            "RCW 59.18.085 — tenant relocation assistance required when landlord requires tenant to vacate for repair / maintenance / abatement"
                .to_string(),
        );
    }

    if input.local_jurisdiction_overlay_applies {
        notes.push(
            "local jurisdiction overlay — Seattle SMC 22.210 or Bellingham Tenant Protections Ordinance may impose additional requirements"
                .to_string(),
        );
    }

    TemporaryRelocationResult {
        compliant: violations.is_empty(),
        violations: violations.clone(),
        citation: citation_for(Regime::Washington),
        notes: notes.clone(),
    }
}

fn check_default(
    _input: &TemporaryRelocationInput,
    _violations: &mut Vec<String>,
    notes: &mut Vec<String>,
) -> TemporaryRelocationResult {
    notes.push(
        "default rule — lease + common-law habitability doctrine; landlord temporary relocation right limited to what lease provides + reasonable accommodation under habitability framework"
            .to_string(),
    );
    notes.push(
        "some municipalities (San Francisco Rent Ordinance § 37.9(a)(11), Long Beach SRTD Ordinance) impose local requirements; check local jurisdiction"
            .to_string(),
    );
    TemporaryRelocationResult {
        compliant: true,
        violations: Vec::new(),
        citation: citation_for(Regime::Default),
        notes: notes.clone(),
    }
}

fn citation_for(regime: Regime) -> &'static str {
    match regime {
        Regime::California => "Cal. Civ. Code §§ 1946.2(d)(2), 1946.2(d)(3), 1942.5(b); SF Rent Ordinance § 37.9(a)(11); Long Beach SRTD Ordinance",
        Regime::NewJersey => "N.J.S.A. §§ 2A:18-61.1(g), 2A:18-61.11",
        Regime::Washington => "RCW 59.18.085; Seattle SMC 22.210; Bellingham Tenant Protections Ordinance",
        Regime::Default => "common-law habitability doctrine + state-specific overlay; check local jurisdiction (SF / Long Beach / Seattle / Bellingham)",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ca_base() -> TemporaryRelocationInput {
        TemporaryRelocationInput {
            regime: Regime::California,
            work_type: WorkType::SubstantialRemodel,
            vacancy_required_30_plus_days: true,
            written_notice_provided: true,
            alternative_housing_offered: false,
            relocation_assistance_paid: true,
            tenant_right_to_return_preserved: true,
            local_jurisdiction_overlay_applies: false,
        }
    }

    fn nj_base() -> TemporaryRelocationInput {
        TemporaryRelocationInput {
            regime: Regime::NewJersey,
            work_type: WorkType::CapitalImprovement,
            vacancy_required_30_plus_days: false,
            written_notice_provided: true,
            alternative_housing_offered: true,
            relocation_assistance_paid: false,
            tenant_right_to_return_preserved: true,
            local_jurisdiction_overlay_applies: false,
        }
    }

    fn wa_base() -> TemporaryRelocationInput {
        TemporaryRelocationInput {
            regime: Regime::Washington,
            work_type: WorkType::CapitalImprovement,
            vacancy_required_30_plus_days: false,
            written_notice_provided: true,
            alternative_housing_offered: false,
            relocation_assistance_paid: true,
            tenant_right_to_return_preserved: true,
            local_jurisdiction_overlay_applies: false,
        }
    }

    fn default_base() -> TemporaryRelocationInput {
        TemporaryRelocationInput {
            regime: Regime::Default,
            work_type: WorkType::CapitalImprovement,
            vacancy_required_30_plus_days: false,
            written_notice_provided: false,
            alternative_housing_offered: false,
            relocation_assistance_paid: false,
            tenant_right_to_return_preserved: true,
            local_jurisdiction_overlay_applies: false,
        }
    }

    #[test]
    fn cosmetic_improvement_no_relocation_required_universal() {
        for regime in [Regime::California, Regime::NewJersey, Regime::Washington, Regime::Default] {
            let mut i = ca_base();
            i.regime = regime;
            i.work_type = WorkType::CosmeticImprovement;
            let r = check(&i);
            assert!(r.compliant);
            assert!(r.notes.iter().any(|n| n.contains("cosmetic improvements") && n.contains("do NOT qualify")));
        }
    }

    #[test]
    fn ca_substantial_remodel_full_compliance_passes() {
        let r = check(&ca_base());
        assert!(r.compliant);
    }

    #[test]
    fn ca_substantial_remodel_without_30_day_vacancy_violation() {
        let mut i = ca_base();
        i.vacancy_required_30_plus_days = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("§ 1946.2(d)(2)") && v.contains("30 days")));
    }

    #[test]
    fn ca_missing_written_notice_violation() {
        let mut i = ca_base();
        i.written_notice_provided = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("§ 1946.2") && v.contains("written notice")));
    }

    #[test]
    fn ca_missing_relocation_or_housing_violation() {
        let mut i = ca_base();
        i.relocation_assistance_paid = false;
        i.alternative_housing_offered = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("§ 1946.2(d)(3)") && v.contains("one-month rent")));
    }

    #[test]
    fn ca_alternative_housing_satisfies_compliance() {
        let mut i = ca_base();
        i.alternative_housing_offered = true;
        i.relocation_assistance_paid = false;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn ca_no_right_to_return_violation() {
        let mut i = ca_base();
        i.tenant_right_to_return_preserved = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("§ 1942.5(b)")));
    }

    #[test]
    fn ca_local_jurisdiction_overlay_note_engaged() {
        let mut i = ca_base();
        i.local_jurisdiction_overlay_applies = true;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("SF Rent Ordinance") || n.contains("Long Beach SRTD")));
    }

    #[test]
    fn nj_full_compliance_passes() {
        let r = check(&nj_base());
        assert!(r.compliant);
    }

    #[test]
    fn nj_missing_written_notice_violation() {
        let mut i = nj_base();
        i.written_notice_provided = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("2A:18-61.1(g)")));
    }

    #[test]
    fn nj_missing_both_housing_and_relocation_violation() {
        let mut i = nj_base();
        i.alternative_housing_offered = false;
        i.relocation_assistance_paid = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("2A:18-61.11")));
    }

    #[test]
    fn nj_relocation_payment_satisfies_compliance() {
        let mut i = nj_base();
        i.alternative_housing_offered = false;
        i.relocation_assistance_paid = true;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn wa_full_compliance_passes() {
        let r = check(&wa_base());
        assert!(r.compliant);
    }

    #[test]
    fn wa_missing_written_notice_violation() {
        let mut i = wa_base();
        i.written_notice_provided = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("RCW 59.18.085")));
    }

    #[test]
    fn wa_missing_relocation_assistance_violation() {
        let mut i = wa_base();
        i.relocation_assistance_paid = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("tenant relocation assistance required")));
    }

    #[test]
    fn wa_local_jurisdiction_overlay_note_engaged() {
        let mut i = wa_base();
        i.local_jurisdiction_overlay_applies = true;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("Seattle SMC 22.210") || n.contains("Bellingham")));
    }

    #[test]
    fn default_compliant_with_lease_plus_common_law() {
        let r = check(&default_base());
        assert!(r.compliant);
        assert!(r.notes.iter().any(|n| n.contains("lease + common-law habitability")));
    }

    #[test]
    fn default_municipal_overlay_note_always_present() {
        let r = check(&default_base());
        assert!(r.notes.iter().any(|n| n.contains("San Francisco") || n.contains("Long Beach") || n.contains("Seattle")));
    }

    #[test]
    fn citation_california_pins_subsections_and_municipal_overlay() {
        let r = check(&ca_base());
        assert!(r.citation.contains("§§ 1946.2(d)(2)"));
        assert!(r.citation.contains("1946.2(d)(3)"));
        assert!(r.citation.contains("1942.5(b)"));
        assert!(r.citation.contains("SF Rent Ordinance"));
        assert!(r.citation.contains("Long Beach SRTD"));
    }

    #[test]
    fn citation_newjersey_pins_subsections() {
        let r = check(&nj_base());
        assert!(r.citation.contains("§§ 2A:18-61.1(g), 2A:18-61.11"));
    }

    #[test]
    fn citation_washington_pins_state_and_local_overlays() {
        let r = check(&wa_base());
        assert!(r.citation.contains("RCW 59.18.085"));
        assert!(r.citation.contains("Seattle SMC 22.210"));
        assert!(r.citation.contains("Bellingham"));
    }

    #[test]
    fn ca_substantial_remodel_30_day_threshold_invariant() {
        let mut i = ca_base();
        i.vacancy_required_30_plus_days = true;
        let r_with = check(&i);
        assert!(r_with.compliant);

        i.vacancy_required_30_plus_days = false;
        let r_without = check(&i);
        assert!(!r_without.compliant);
    }

    #[test]
    fn cosmetic_improvement_path_bypasses_all_other_compliance_checks() {
        let mut i = TemporaryRelocationInput {
            regime: Regime::California,
            work_type: WorkType::CosmeticImprovement,
            vacancy_required_30_plus_days: false,
            written_notice_provided: false,
            alternative_housing_offered: false,
            relocation_assistance_paid: false,
            tenant_right_to_return_preserved: false,
            local_jurisdiction_overlay_applies: false,
        };
        let r = check(&i);
        assert!(r.compliant, "cosmetic improvements bypass all other checks");
        i.regime = Regime::Default;
        let r2 = check(&i);
        assert!(r2.compliant);
    }

    #[test]
    fn nj_no_return_right_note_engaged() {
        let mut i = nj_base();
        i.tenant_right_to_return_preserved = false;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("2A:18-61.1(g) permits")));
    }

    #[test]
    fn ca_multiple_violations_accumulate() {
        let mut i = ca_base();
        i.written_notice_provided = false;
        i.relocation_assistance_paid = false;
        i.alternative_housing_offered = false;
        i.tenant_right_to_return_preserved = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.len() >= 3);
    }

    #[test]
    fn nj_unique_anti_eviction_act_invariant() {
        let mut i_nj = nj_base();
        i_nj.written_notice_provided = false;
        let r_nj = check(&i_nj);
        assert!(r_nj.violations.iter().any(|v| v.contains("2A:18-61.1(g)")));

        for regime in [Regime::California, Regime::Washington, Regime::Default] {
            let mut i = nj_base();
            i.regime = regime;
            let r = check(&i);
            assert!(!r.violations.iter().any(|v| v.contains("2A:18-61.1(g)")));
        }
    }

    #[test]
    fn ca_30_day_threshold_unique_to_california() {
        let mut i_ca = ca_base();
        i_ca.vacancy_required_30_plus_days = false;
        let r_ca = check(&i_ca);
        assert!(!r_ca.compliant);

        for regime in [Regime::NewJersey, Regime::Washington, Regime::Default] {
            let mut i = ca_base();
            i.regime = regime;
            i.vacancy_required_30_plus_days = false;
            i.relocation_assistance_paid = true;
            i.alternative_housing_offered = true;
            i.written_notice_provided = true;
            let r = check(&i);
            assert!(!r.violations.iter().any(|v| v.contains("§ 1946.2(d)(2)")));
        }
    }
}
