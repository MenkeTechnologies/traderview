//! Tenant right to choose renters insurance provider when
//! landlord mandates coverage — anti-tying framework. When a
//! landlord requires tenant to carry renters insurance, may the
//! landlord also dictate WHICH insurer the tenant must use? In
//! virtually every state, the answer is NO: the landlord may
//! require coverage at specified minimum levels but may not
//! mandate a specific insurer (especially a landlord-affiliated
//! provider). Distinct from `renters_insurance` (general renters
//! insurance framework + coverage minimums) and `rental_junk_fee_
//! transparency` (which addresses non-rent fee transparency).
//!
//! Trader-landlord operational concern when landlord (or an
//! affiliate / preferred-vendor program) earns commissions on
//! tenant-purchased renters insurance and the lease appears to
//! mandate that specific provider. Antitying violations expose
//! landlord to state UDAP claims, treble damages, attorney fees,
//! and potential FTC § 5 unfair-or-deceptive-practices liability.
//!
//! **Three regimes**:
//!
//! **California — Cal. Ins. Code (anti-tying doctrine) + Cal.
//! Civ. Code § 1942.6 (renters insurance)**. Landlord MAY require
//! tenant to carry renters insurance and MAY specify minimum
//! coverage levels in the lease, but MAY NOT mandate a specific
//! insurer. Tenant retains right to choose any licensed insurer
//! whose policy meets the lease's coverage requirements.
//! Landlord may RECOMMEND a list of providers without mandating
//! one. Antitying-arrangement violations void the mandate and
//! expose landlord to claims under California UDAP (Cal. Bus. &
//! Prof. Code § 17200) and Cal. Civ. Code § 1750 et seq.
//! (Consumers Legal Remedies Act).
//!
//! **New York — N.Y. Gen. Bus. Law § 349 (Deceptive Acts and
//! Practices) + N.Y. Ins. Law § 2502**. Landlord may require
//! renters insurance and specify minimum coverage but may not
//! tie the requirement to a specific provider. Antitying
//! violations actionable under GBL § 349 with $50/$1,000 minimum
//! statutory damages plus attorney fees + treble damages on
//! willful violation.
//!
//! **Default — common-law anti-tying + state UDAP**. Most states
//! follow the same doctrine — landlord requirement permitted,
//! mandate-of-specific-provider prohibited as deceptive or
//! coercive practice. Federal FTC § 5 (15 U.S.C. § 45) applies
//! generally to landlord-as-insurance-agent arrangements. State
//! UDAP statutes (47 states + DC) provide private right of
//! action with varying penalty structures.
//!
//! Citations: Cal. Ins. Code + Cal. Civ. Code § 1942.6 (renters
//! insurance); Cal. Bus. & Prof. Code § 17200 (California UDAP /
//! Unfair Competition Law); Cal. Civ. Code § 1750 et seq.
//! (Consumers Legal Remedies Act); N.Y. Gen. Bus. Law § 349
//! (Deceptive Acts and Practices); N.Y. Ins. Law § 2502 (limited
//! license / insurance agent regulation); 15 U.S.C. § 45 (FTC
//! Act § 5 UDAP); state UDAP statutes (47 states + DC private
//! right of action).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    California,
    NewYork,
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MandatoryRentersInsuranceInput {
    pub regime: Regime,
    /// Whether the lease requires the tenant to carry renters
    /// insurance.
    pub landlord_requires_renters_insurance: bool,
    /// Whether the landlord mandates a SPECIFIC insurer rather
    /// than merely specifying coverage minimums.
    pub landlord_mandated_specific_provider: bool,
    /// Whether the mandated provider is the landlord or a
    /// landlord-affiliated entity (heightens anti-tying concern).
    pub landlord_or_affiliate_is_provider: bool,
    /// Whether the lease specifies coverage REQUIREMENTS (minimum
    /// liability limits, named perils) — landlord may specify
    /// these without violating anti-tying.
    pub lease_specifies_coverage_minimums: bool,
    /// Whether the tenant offered (or could offer) a certificate
    /// of insurance from a chosen licensed insurer meeting the
    /// coverage minimums.
    pub tenant_offered_certificate_from_chosen_provider: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MandatoryRentersInsuranceResult {
    pub requirement_enforceable: bool,
    pub anti_tying_violation: bool,
    /// Whether the tenant's right to choose any compliant
    /// licensed insurer is engaged.
    pub tenant_right_to_choose_provider_engaged: bool,
    /// Whether the landlord's coverage-minimum specification
    /// itself is permitted (always YES — separate from
    /// provider mandate).
    pub coverage_minimum_specification_permitted: bool,
    /// Whether heightened scrutiny applies because landlord or
    /// affiliate would earn commissions on tenant-purchased
    /// insurance.
    pub affiliate_financial_interest_heightens_scrutiny: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &MandatoryRentersInsuranceInput) -> MandatoryRentersInsuranceResult {
    match input.regime {
        Regime::California => check_california(input),
        Regime::NewYork => check_new_york(input),
        Regime::Default => check_default(input),
    }
}

fn check_california(input: &MandatoryRentersInsuranceInput) -> MandatoryRentersInsuranceResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "California — landlord may RECOMMEND a list of providers without mandating one; recommendation is not anti-tying"
            .to_string(),
    ];

    if input.landlord_mandated_specific_provider {
        violations.push(
            "Cal. Ins. Code + Cal. Civ. Code § 1942.6 — landlord MAY require renters insurance and MAY specify minimum coverage but MAY NOT mandate a specific insurer; tenant retains right to choose any licensed insurer whose policy meets lease coverage requirements"
                .to_string(),
        );
        violations.push(
            "Cal. Bus. & Prof. Code § 17200 (California UDAP / Unfair Competition Law) — landlord mandate of specific provider may constitute unlawful business practice; private right of action with injunctive + restitution remedies"
                .to_string(),
        );
        if input.landlord_or_affiliate_is_provider {
            violations.push(
                "Cal. Civ. Code § 1750 et seq. (Consumers Legal Remedies Act) — heightened scrutiny when landlord or affiliate would earn commissions on tenant-purchased insurance; treble damages + attorney fees + injunctive relief"
                    .to_string(),
            );
        }
    }

    let enforceable =
        input.landlord_requires_renters_insurance && !input.landlord_mandated_specific_provider;
    let anti_tying = input.landlord_mandated_specific_provider;
    let right_to_choose = input.landlord_requires_renters_insurance;
    let affiliate_scrutiny =
        input.landlord_mandated_specific_provider && input.landlord_or_affiliate_is_provider;

    MandatoryRentersInsuranceResult {
        requirement_enforceable: enforceable,
        anti_tying_violation: anti_tying,
        tenant_right_to_choose_provider_engaged: right_to_choose,
        coverage_minimum_specification_permitted: input.lease_specifies_coverage_minimums,
        affiliate_financial_interest_heightens_scrutiny: affiliate_scrutiny,
        violations,
        citation:
            "Cal. Ins. Code; Cal. Civ. Code §§ 1942.6, 1750 et seq.; Cal. Bus. & Prof. Code § 17200",
        notes,
    }
}

fn check_new_york(input: &MandatoryRentersInsuranceInput) -> MandatoryRentersInsuranceResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "New York — landlord may specify coverage requirements (minimum liability limits) without violating anti-tying"
            .to_string(),
    ];

    if input.landlord_mandated_specific_provider {
        violations.push(
            "N.Y. Gen. Bus. Law § 349 (Deceptive Acts and Practices) — landlord mandate of specific renters-insurance provider is deceptive practice; minimum $50 statutory damages per violation + $1,000 statutory damages cap for willful violation + treble damages + attorney fees"
                .to_string(),
        );
        violations.push(
            "N.Y. Ins. Law § 2502 — limited license / insurance agent regulation; landlord acting as de facto insurance agent without licensing may violate § 2502"
                .to_string(),
        );
    }

    let enforceable =
        input.landlord_requires_renters_insurance && !input.landlord_mandated_specific_provider;
    let anti_tying = input.landlord_mandated_specific_provider;
    let right_to_choose = input.landlord_requires_renters_insurance;
    let affiliate_scrutiny =
        input.landlord_mandated_specific_provider && input.landlord_or_affiliate_is_provider;

    MandatoryRentersInsuranceResult {
        requirement_enforceable: enforceable,
        anti_tying_violation: anti_tying,
        tenant_right_to_choose_provider_engaged: right_to_choose,
        coverage_minimum_specification_permitted: input.lease_specifies_coverage_minimums,
        affiliate_financial_interest_heightens_scrutiny: affiliate_scrutiny,
        violations,
        citation: "N.Y. Gen. Bus. Law § 349; N.Y. Ins. Law § 2502",
        notes,
    }
}

fn check_default(input: &MandatoryRentersInsuranceInput) -> MandatoryRentersInsuranceResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "default rule — most states follow common-law anti-tying doctrine + state UDAP framework; landlord may require renters insurance and specify coverage minimums but may not mandate a specific provider"
            .to_string(),
        "15 U.S.C. § 45 (FTC Act § 5 UDAP) — federal Unfair or Deceptive Acts or Practices doctrine applies to landlord-as-insurance-agent arrangements; state UDAP statutes (47 states + DC) provide private right of action"
            .to_string(),
    ];

    if input.landlord_mandated_specific_provider {
        violations.push(
            "common-law anti-tying + state UDAP — landlord mandate of specific renters-insurance provider may be unenforceable under state Deceptive Acts and Practices statute; landlord exposes itself to state-specific UDAP penalties and federal FTC § 5 liability"
                .to_string(),
        );
    }

    let enforceable =
        input.landlord_requires_renters_insurance && !input.landlord_mandated_specific_provider;
    let anti_tying = input.landlord_mandated_specific_provider;
    let right_to_choose = input.landlord_requires_renters_insurance;
    let affiliate_scrutiny =
        input.landlord_mandated_specific_provider && input.landlord_or_affiliate_is_provider;

    MandatoryRentersInsuranceResult {
        requirement_enforceable: enforceable,
        anti_tying_violation: anti_tying,
        tenant_right_to_choose_provider_engaged: right_to_choose,
        coverage_minimum_specification_permitted: input.lease_specifies_coverage_minimums,
        affiliate_financial_interest_heightens_scrutiny: affiliate_scrutiny,
        violations,
        citation:
            "common-law anti-tying + state-specific UDAP statutes + 15 U.S.C. § 45 (FTC Act § 5)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ca_compliant() -> MandatoryRentersInsuranceInput {
        MandatoryRentersInsuranceInput {
            regime: Regime::California,
            landlord_requires_renters_insurance: true,
            landlord_mandated_specific_provider: false,
            landlord_or_affiliate_is_provider: false,
            lease_specifies_coverage_minimums: true,
            tenant_offered_certificate_from_chosen_provider: true,
        }
    }

    fn ny_compliant() -> MandatoryRentersInsuranceInput {
        let mut i = ca_compliant();
        i.regime = Regime::NewYork;
        i
    }

    fn default_compliant() -> MandatoryRentersInsuranceInput {
        let mut i = ca_compliant();
        i.regime = Regime::Default;
        i
    }

    #[test]
    fn ca_clean_requirement_enforceable() {
        let r = check(&ca_compliant());
        assert!(r.requirement_enforceable);
        assert!(!r.anti_tying_violation);
        assert!(r.tenant_right_to_choose_provider_engaged);
        assert!(r.coverage_minimum_specification_permitted);
    }

    #[test]
    fn ca_provider_mandate_triggers_anti_tying_violation() {
        let mut i = ca_compliant();
        i.landlord_mandated_specific_provider = true;
        let r = check(&i);
        assert!(!r.requirement_enforceable);
        assert!(r.anti_tying_violation);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 1942.6") && v.contains("MAY NOT mandate")));
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 17200") && v.contains("UDAP")));
    }

    #[test]
    fn ca_affiliate_provider_heightens_scrutiny() {
        let mut i = ca_compliant();
        i.landlord_mandated_specific_provider = true;
        i.landlord_or_affiliate_is_provider = true;
        let r = check(&i);
        assert!(r.affiliate_financial_interest_heightens_scrutiny);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 1750") && v.contains("Consumers Legal Remedies Act")));
    }

    #[test]
    fn ca_affiliate_without_mandate_no_heightened_scrutiny() {
        let mut i = ca_compliant();
        i.landlord_or_affiliate_is_provider = true;
        let r = check(&i);
        assert!(!r.affiliate_financial_interest_heightens_scrutiny);
    }

    #[test]
    fn ca_recommendation_note_describes_permitted_path() {
        let r = check(&ca_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("RECOMMEND a list of providers")));
    }

    #[test]
    fn ca_citation_pins_subsections() {
        let r = check(&ca_compliant());
        assert!(r.citation.contains("§§ 1942.6, 1750 et seq."));
        assert!(r.citation.contains("§ 17200"));
    }

    #[test]
    fn ny_clean_requirement_enforceable() {
        let r = check(&ny_compliant());
        assert!(r.requirement_enforceable);
        assert!(!r.anti_tying_violation);
    }

    #[test]
    fn ny_provider_mandate_triggers_gbl_349_violation() {
        let mut i = ny_compliant();
        i.landlord_mandated_specific_provider = true;
        let r = check(&i);
        assert!(!r.requirement_enforceable);
        assert!(r.anti_tying_violation);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 349") && v.contains("treble damages")));
    }

    #[test]
    fn ny_provider_mandate_also_triggers_ins_2502() {
        let mut i = ny_compliant();
        i.landlord_mandated_specific_provider = true;
        let r = check(&i);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 2502") && v.contains("de facto insurance agent")));
    }

    #[test]
    fn ny_citation_pins_gbl_349_and_ins_2502() {
        let r = check(&ny_compliant());
        assert!(r.citation.contains("§ 349"));
        assert!(r.citation.contains("§ 2502"));
    }

    #[test]
    fn default_clean_requirement_enforceable() {
        let r = check(&default_compliant());
        assert!(r.requirement_enforceable);
    }

    #[test]
    fn default_provider_mandate_triggers_udap_violation() {
        let mut i = default_compliant();
        i.landlord_mandated_specific_provider = true;
        let r = check(&i);
        assert!(!r.requirement_enforceable);
        assert!(r.anti_tying_violation);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("common-law anti-tying") && v.contains("UDAP")));
    }

    #[test]
    fn default_ftc_act_section_5_note_present() {
        let r = check(&default_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("15 U.S.C. § 45") && n.contains("47 states + DC")));
    }

    #[test]
    fn default_citation_references_ftc_section_5() {
        let r = check(&default_compliant());
        assert!(r.citation.contains("15 U.S.C. § 45"));
        assert!(r.citation.contains("FTC Act § 5"));
    }

    #[test]
    fn no_requirement_no_enforceability_engagement() {
        let mut i = ca_compliant();
        i.landlord_requires_renters_insurance = false;
        let r = check(&i);
        assert!(!r.requirement_enforceable);
        assert!(!r.tenant_right_to_choose_provider_engaged);
    }

    #[test]
    fn coverage_minimum_specification_permitted_invariant() {
        let r = check(&ca_compliant());
        assert!(r.coverage_minimum_specification_permitted);
    }

    #[test]
    fn coverage_minimum_independent_of_provider_mandate() {
        let mut i = ca_compliant();
        i.landlord_mandated_specific_provider = true;
        i.lease_specifies_coverage_minimums = true;
        let r = check(&i);
        assert!(r.coverage_minimum_specification_permitted);
        assert!(r.anti_tying_violation);
    }

    #[test]
    fn three_regimes_routed_correctly() {
        for regime in [Regime::California, Regime::NewYork, Regime::Default] {
            let mut i = ca_compliant();
            i.regime = regime;
            let r = check(&i);
            let _ = r.requirement_enforceable;
            assert!(!r.citation.is_empty());
        }
    }

    #[test]
    fn anti_tying_violation_invariant_across_regimes() {
        for regime in [Regime::California, Regime::NewYork, Regime::Default] {
            let mut i = ca_compliant();
            i.regime = regime;
            i.landlord_mandated_specific_provider = true;
            let r = check(&i);
            assert!(
                r.anti_tying_violation,
                "regime {:?} should flag anti-tying violation",
                regime
            );
            assert!(!r.requirement_enforceable);
        }
    }

    #[test]
    fn clean_compliance_invariant_across_regimes() {
        for regime in [Regime::California, Regime::NewYork, Regime::Default] {
            let mut i = ca_compliant();
            i.regime = regime;
            let r = check(&i);
            assert!(r.requirement_enforceable);
            assert!(!r.anti_tying_violation);
        }
    }

    #[test]
    fn affiliate_scrutiny_only_when_mandate_and_affiliate() {
        for mandate in [false, true] {
            for affiliate in [false, true] {
                let mut i = ca_compliant();
                i.landlord_mandated_specific_provider = mandate;
                i.landlord_or_affiliate_is_provider = affiliate;
                let r = check(&i);
                assert_eq!(
                    r.affiliate_financial_interest_heightens_scrutiny,
                    mandate && affiliate
                );
            }
        }
    }

    #[test]
    fn ca_clean_no_violations() {
        let r = check(&ca_compliant());
        assert!(r.violations.is_empty());
    }

    #[test]
    fn ny_clean_no_violations() {
        let r = check(&ny_compliant());
        assert!(r.violations.is_empty());
    }

    #[test]
    fn default_clean_no_violations() {
        let r = check(&default_compliant());
        assert!(r.violations.is_empty());
    }

    #[test]
    fn ca_multiple_violations_simultaneous_with_affiliate() {
        let mut i = ca_compliant();
        i.landlord_mandated_specific_provider = true;
        i.landlord_or_affiliate_is_provider = true;
        let r = check(&i);
        assert_eq!(r.violations.len(), 3);
    }

    #[test]
    fn ny_provider_mandate_two_violations_simultaneous() {
        let mut i = ny_compliant();
        i.landlord_mandated_specific_provider = true;
        let r = check(&i);
        assert_eq!(r.violations.len(), 2);
    }

    #[test]
    fn ca_recommendation_path_does_not_trigger_violation() {
        let r = check(&ca_compliant());
        assert!(!r.anti_tying_violation);
        assert!(r.violations.is_empty());
    }
}
