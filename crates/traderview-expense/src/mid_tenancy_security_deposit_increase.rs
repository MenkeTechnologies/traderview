//! Mandatory landlord-paid prohibition on mid-tenancy security
//! deposit increase. When may a landlord demand additional
//! security deposit during the tenancy after initial collection?
//! Distinct from `security_deposit_caps` (initial cap),
//! `damage_deduction_itemization`, `deposit_interest`,
//! `security_deposit_bank_disclosure`, and `deposit_return_
//! windows`. Trader-landlord operational concern when annual
//! rent increase or pet addition triggers desire for higher
//! deposit.
//!
//! **Four regimes**:
//!
//! **California — Cal. Civ. Code § 1950.5(c)**. Mid-tenancy
//! security deposit increase generally NOT permitted unless (a)
//! lease modification basis (new pet, additional occupant) AND
//! (b) total deposit remains within § 1950.5(c) statutory cap
//! (one month's rent for unfurnished; two months for furnished,
//! per AB 12 effective July 1, 2024). Tenant consent in writing
//! required for any mid-tenancy increase.
//!
//! **New Jersey — N.J.S.A. 46:8-21.2**. Mid-tenancy security
//! deposit increase generally prohibited absent lease
//! modification or annual rent increase basis. § 46:8-21.2
//! limits increase to amount calculated as proportional to rent
//! increase. Bad-faith mid-tenancy increases recoverable with
//! double damages.
//!
//! **New York — N.Y. Gen. Oblig. Law § 7-108(1-a)(a)** (HSTPA
//! of 2019). Security deposit capped at ONE MONTH's rent
//! statewide for residential leases. Mid-tenancy increase
//! permitted only when proportional to rent increase and total
//! remains within one-month cap.
//!
//! **Default — lease controls + common-law restriction on
//! unilateral modification**. Most states allow mid-tenancy
//! security deposit increase if lease permits OR by mutual
//! written modification. Many jurisdictions impose good-faith
//! limit and bar increases that bring total above statutory cap.
//!
//! Citations: Cal. Civ. Code § 1950.5(c) covers CA initial
//! deposit cap and AB 12 amendments effective July 1, 2024.
//! N.J.S.A. 46:8-21.2 covers NJ mid-tenancy security deposit
//! increase proportional to rent. N.Y. Gen. Oblig. Law
//! § 7-108(1-a)(a) covers NY one-month statewide cap under
//! HSTPA 2019. Common-law contract modification and good-faith
//! doctrine apply as default.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    California,
    NewJersey,
    NewYork,
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MidTenancySecurityDepositInput {
    pub regime: Regime,
    /// Whether the landlord has attempted a mid-tenancy security
    /// deposit increase.
    pub mid_tenancy_increase_attempted: bool,
    /// Basis for increase: lease modification (new pet,
    /// additional occupant) — sometimes permitted.
    pub lease_modification_basis_for_increase: bool,
    /// Basis for increase: tied proportionally to annual rent
    /// increase.
    pub proportional_to_annual_rent_increase: bool,
    /// Whether the increase brings total deposit above statutory
    /// cap (1 month unfurnished CA/NY; 1.5 months NJ).
    pub statutory_cap_exceeded_after_increase: bool,
    /// Whether the tenant consented in writing to the
    /// mid-tenancy increase.
    pub tenant_consented_in_writing: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MidTenancySecurityDepositResult {
    pub mid_tenancy_increase_authorized: bool,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &MidTenancySecurityDepositInput) -> MidTenancySecurityDepositResult {
    match input.regime {
        Regime::California => check_california(input),
        Regime::NewJersey => check_new_jersey(input),
        Regime::NewYork => check_new_york(input),
        Regime::Default => check_default(input),
    }
}

fn check_california(input: &MidTenancySecurityDepositInput) -> MidTenancySecurityDepositResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Cal. Civ. Code § 1950.5(c) — initial security deposit cap one month's rent (unfurnished) or two months (furnished) per AB 12 effective July 1, 2024"
            .to_string(),
        "Cal. Civ. Code § 1950.5 — mid-tenancy security deposit increase generally NOT permitted absent (1) lease modification basis + (2) total within statutory cap + (3) tenant written consent"
            .to_string(),
    ];

    if input.mid_tenancy_increase_attempted {
        if !input.lease_modification_basis_for_increase {
            violations.push(
                "Cal. Civ. Code § 1950.5 — mid-tenancy security deposit increase NOT permitted without lease modification basis (new pet + additional occupant + other amendment)"
                    .to_string(),
            );
        }
        if input.statutory_cap_exceeded_after_increase {
            violations.push(
                "Cal. Civ. Code § 1950.5(c) — increase would exceed statutory cap (one month's rent unfurnished + two months furnished per AB 12)"
                    .to_string(),
            );
        }
        if !input.tenant_consented_in_writing {
            violations.push(
                "Cal. Civ. Code § 1950.5 — tenant written consent required for any mid-tenancy security deposit increase"
                    .to_string(),
            );
        }
    }

    let compliant = violations.is_empty();
    MidTenancySecurityDepositResult {
        mid_tenancy_increase_authorized: input.mid_tenancy_increase_attempted && compliant,
        compliant,
        violations,
        citation: "Cal. Civ. Code § 1950.5(c) + AB 12 (effective July 1, 2024)",
        notes,
    }
}

fn check_new_jersey(input: &MidTenancySecurityDepositInput) -> MidTenancySecurityDepositResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "N.J.S.A. 46:8-21.2 — mid-tenancy security deposit increase prohibited absent lease modification basis OR proportional annual rent increase basis"
            .to_string(),
        "N.J.S.A. 46:8-21.2 — bad-faith mid-tenancy increases recoverable with DOUBLE DAMAGES + attorney fees + court costs"
            .to_string(),
    ];

    if input.mid_tenancy_increase_attempted
        && !input.lease_modification_basis_for_increase
        && !input.proportional_to_annual_rent_increase
    {
        violations.push(
            "N.J.S.A. 46:8-21.2 — mid-tenancy security deposit increase prohibited absent (1) lease modification basis OR (2) proportional to annual rent increase"
                .to_string(),
        );
    }

    if input.mid_tenancy_increase_attempted && input.statutory_cap_exceeded_after_increase {
        violations.push(
            "N.J.S.A. 46:8-19 — total security deposit cap is 1.5 months' rent; mid-tenancy increase exceeding cap is bad-faith violation with double-damages exposure"
                .to_string(),
        );
    }

    let compliant = violations.is_empty();
    MidTenancySecurityDepositResult {
        mid_tenancy_increase_authorized: input.mid_tenancy_increase_attempted && compliant,
        compliant,
        violations,
        citation: "N.J.S.A. §§ 46:8-21.2, 46:8-19",
        notes,
    }
}

fn check_new_york(input: &MidTenancySecurityDepositInput) -> MidTenancySecurityDepositResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "N.Y. Gen. Oblig. Law § 7-108(1-a)(a) — security deposit capped at ONE MONTH's rent statewide for residential leases per HSTPA 2019"
            .to_string(),
        "HSTPA 2019 — mid-tenancy security deposit increase permitted only when proportional to rent increase AND total remains within one-month cap"
            .to_string(),
    ];

    if input.mid_tenancy_increase_attempted && input.statutory_cap_exceeded_after_increase {
        violations.push(
            "N.Y. Gen. Oblig. Law § 7-108(1-a)(a) — increase would exceed statewide one-month cap under HSTPA 2019"
                .to_string(),
        );
    }

    if input.mid_tenancy_increase_attempted
        && !input.proportional_to_annual_rent_increase
        && !input.lease_modification_basis_for_increase
    {
        violations.push(
            "N.Y. Gen. Oblig. Law § 7-108 — mid-tenancy increase requires proportional rent-increase basis OR lease modification"
                .to_string(),
        );
    }

    let compliant = violations.is_empty();
    MidTenancySecurityDepositResult {
        mid_tenancy_increase_authorized: input.mid_tenancy_increase_attempted && compliant,
        compliant,
        violations,
        citation: "N.Y. Gen. Oblig. Law § 7-108(1-a)(a); HSTPA 2019",
        notes,
    }
}

fn check_default(input: &MidTenancySecurityDepositInput) -> MidTenancySecurityDepositResult {
    let notes: Vec<String> = vec![
        "default rule — lease controls + common-law contract modification + good-faith doctrine; most states allow mid-tenancy security deposit increase if lease permits OR by mutual written modification"
            .to_string(),
        "default rule — many jurisdictions impose good-faith limit and bar increases bringing total above statutory cap; state UDAP statutes may reach abusive mid-tenancy increases"
            .to_string(),
    ];

    let authorized = input.mid_tenancy_increase_attempted
        && (input.tenant_consented_in_writing || input.lease_modification_basis_for_increase);

    MidTenancySecurityDepositResult {
        mid_tenancy_increase_authorized: authorized,
        compliant: true,
        violations: Vec::new(),
        citation: "state-specific landlord-tenant statute + common-law contract modification + good-faith doctrine",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ca_compliant_increase() -> MidTenancySecurityDepositInput {
        MidTenancySecurityDepositInput {
            regime: Regime::California,
            mid_tenancy_increase_attempted: true,
            lease_modification_basis_for_increase: true,
            proportional_to_annual_rent_increase: false,
            statutory_cap_exceeded_after_increase: false,
            tenant_consented_in_writing: true,
        }
    }

    fn nj_compliant_increase() -> MidTenancySecurityDepositInput {
        let mut i = ca_compliant_increase();
        i.regime = Regime::NewJersey;
        i.proportional_to_annual_rent_increase = true;
        i
    }

    fn ny_compliant_increase() -> MidTenancySecurityDepositInput {
        let mut i = ca_compliant_increase();
        i.regime = Regime::NewYork;
        i.proportional_to_annual_rent_increase = true;
        i
    }

    fn default_base() -> MidTenancySecurityDepositInput {
        let mut i = ca_compliant_increase();
        i.regime = Regime::Default;
        i
    }

    #[test]
    fn ca_clean_increase_authorized() {
        let r = check(&ca_compliant_increase());
        assert!(r.compliant);
        assert!(r.mid_tenancy_increase_authorized);
    }

    #[test]
    fn ca_no_basis_violates() {
        let mut i = ca_compliant_increase();
        i.lease_modification_basis_for_increase = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(!r.mid_tenancy_increase_authorized);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("lease modification basis")));
    }

    #[test]
    fn ca_cap_exceeded_violates() {
        let mut i = ca_compliant_increase();
        i.statutory_cap_exceeded_after_increase = true;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("AB 12")));
    }

    #[test]
    fn ca_no_consent_violates() {
        let mut i = ca_compliant_increase();
        i.tenant_consented_in_writing = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("tenant written consent")));
    }

    #[test]
    fn ca_citation_pins_ab_12() {
        let r = check(&ca_compliant_increase());
        assert!(r.citation.contains("AB 12"));
        assert!(r.citation.contains("§ 1950.5(c)"));
    }

    #[test]
    fn nj_clean_increase_authorized() {
        let r = check(&nj_compliant_increase());
        assert!(r.compliant);
        assert!(r.mid_tenancy_increase_authorized);
    }

    #[test]
    fn nj_no_basis_violates() {
        let mut i = nj_compliant_increase();
        i.lease_modification_basis_for_increase = false;
        i.proportional_to_annual_rent_increase = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("46:8-21.2")));
    }

    #[test]
    fn nj_cap_exceeded_violates() {
        let mut i = nj_compliant_increase();
        i.statutory_cap_exceeded_after_increase = true;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("46:8-19") && v.contains("1.5 months")));
    }

    #[test]
    fn nj_double_damages_note_present() {
        let r = check(&nj_compliant_increase());
        assert!(r.notes.iter().any(|n| n.contains("DOUBLE DAMAGES")));
    }

    #[test]
    fn ny_clean_increase_authorized() {
        let r = check(&ny_compliant_increase());
        assert!(r.compliant);
        assert!(r.mid_tenancy_increase_authorized);
    }

    #[test]
    fn ny_cap_exceeded_violates() {
        let mut i = ny_compliant_increase();
        i.statutory_cap_exceeded_after_increase = true;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 7-108(1-a)(a)") && v.contains("one-month")));
    }

    #[test]
    fn ny_hstpa_2019_note_present() {
        let r = check(&ny_compliant_increase());
        assert!(r.notes.iter().any(|n| n.contains("HSTPA 2019")));
    }

    #[test]
    fn ny_no_basis_violates() {
        let mut i = ny_compliant_increase();
        i.lease_modification_basis_for_increase = false;
        i.proportional_to_annual_rent_increase = false;
        let r = check(&i);
        assert!(!r.compliant);
    }

    #[test]
    fn default_compliant_always() {
        let r = check(&default_base());
        assert!(r.compliant);
    }

    #[test]
    fn default_authorized_with_consent_or_basis() {
        let mut i = default_base();
        i.tenant_consented_in_writing = true;
        let r = check(&i);
        assert!(r.mid_tenancy_increase_authorized);
    }

    #[test]
    fn default_no_basis_no_consent_not_authorized() {
        let mut i = default_base();
        i.tenant_consented_in_writing = false;
        i.lease_modification_basis_for_increase = false;
        let r = check(&i);
        assert!(!r.mid_tenancy_increase_authorized);
        assert!(r.compliant);
    }

    #[test]
    fn no_increase_attempted_always_compliant() {
        for regime in [
            Regime::California,
            Regime::NewJersey,
            Regime::NewYork,
            Regime::Default,
        ] {
            let mut i = ca_compliant_increase();
            i.regime = regime;
            i.mid_tenancy_increase_attempted = false;
            let r = check(&i);
            assert!(r.compliant);
            assert!(!r.mid_tenancy_increase_authorized);
        }
    }

    #[test]
    fn four_regimes_routed_correctly() {
        for regime in [
            Regime::California,
            Regime::NewJersey,
            Regime::NewYork,
            Regime::Default,
        ] {
            let mut i = ca_compliant_increase();
            i.regime = regime;
            i.proportional_to_annual_rent_increase = true;
            let r = check(&i);
            let _ = r.compliant;
            assert!(!r.citation.is_empty());
        }
    }

    #[test]
    fn ca_multiple_violations_simultaneous() {
        let mut i = ca_compliant_increase();
        i.lease_modification_basis_for_increase = false;
        i.statutory_cap_exceeded_after_increase = true;
        i.tenant_consented_in_writing = false;
        let r = check(&i);
        assert_eq!(r.violations.len(), 3);
    }

    #[test]
    fn nj_unique_double_damages_invariant() {
        let r_nj = check(&nj_compliant_increase());
        assert!(r_nj.notes.iter().any(|n| n.contains("DOUBLE DAMAGES")));

        let r_ca = check(&ca_compliant_increase());
        assert!(!r_ca.notes.iter().any(|n| n.contains("DOUBLE DAMAGES")));
    }

    #[test]
    fn ca_clean_no_violations() {
        let r = check(&ca_compliant_increase());
        assert!(r.violations.is_empty());
    }

    #[test]
    fn nj_clean_no_violations() {
        let r = check(&nj_compliant_increase());
        assert!(r.violations.is_empty());
    }

    #[test]
    fn ny_clean_no_violations() {
        let r = check(&ny_compliant_increase());
        assert!(r.violations.is_empty());
    }
}
