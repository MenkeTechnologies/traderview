//! Broker fee allocation between landlord and tenant — who pays
//! the rental broker's commission when a rental agreement is
//! executed? Trader-landlord operational concern in markets where
//! broker fees range from one month's rent to 15% of annual rent.
//!
//! Distinct from `application_fees` (pre-tenancy screening fees),
//! `non_refundable_cleaning_fees` (move-in non-refundable charges),
//! `pet_fees` (pet-specific charges), and `lease_disclosures`
//! (general lease disclosures). This module addresses ONLY the
//! BROKER FEE allocation question — who legally owes the broker's
//! commission when the broker is engaged in connection with the
//! rental.
//!
//! Two regimes:
//!
//! **New York City — FARE Act (NYC Local Law 119, eff. June 11,
//! 2025)**. PARTY-WHO-HIRED-PAYS rule. The party who retains the
//! broker is the party responsible for paying the broker fee.
//! Landlords (or their agents) MAY NOT impose or collect a broker
//! fee from a tenant for the services of a broker the landlord
//! engaged. Landlords MUST disclose other rental-related fees in
//! the listing and rental agreement. Enforced by NYC DCWP
//! (Department of Consumer and Worker Protection) via civil
//! penalty or civil action. Listing brokers who publish with
//! landlord's permission are bound by the same rule. Tenants
//! retain the right to hire their own broker (in which case the
//! tenant pays the broker they hired).
//!
//! **Default — lease + market practice**. Most US jurisdictions
//! lack a statutory rule governing broker fee allocation; the
//! lease + market custom controls. In tight rental markets
//! (NYC pre-FARE, San Francisco, Boston) tenants traditionally
//! paid 1 month's rent or 15% of annual rent as broker fee.
//! Boston enacted Ordinance 17-2024 (Boston Municipal Code § 9-6
//! amendments) imposing similar disclosure requirements; treated
//! as Default + municipal disclosure layer.
//!
//! Citations: NYC Local Law 119 / Admin. Code § 20-699.20
//! (FARE Act); NYC Admin. Code § 20-699.21 (disclosure of other
//! fees); NYC Admin. Code § 20-699.22 (DCWP enforcement); Boston
//! Ord. 17-2024 (Boston Municipal Code § 9-6); common-law
//! contract + state broker license statutes elsewhere.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    /// NYC FARE Act (Local Law 119, eff. June 11, 2025).
    NewYorkCityFareAct,
    /// All other jurisdictions — lease + market practice.
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PartyResponsibleForFee {
    Landlord,
    Tenant,
    None,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BrokerFeeAllocationInput {
    pub regime: Regime,
    /// Whether the landlord (or landlord's agent) engaged the
    /// broker. Drives the NYC FARE Act party-who-hired-pays rule.
    pub landlord_hired_broker: bool,
    /// Whether the tenant separately retained their own broker.
    pub tenant_hired_own_broker: bool,
    /// Whether the landlord (or landlord's agent) is attempting to
    /// charge or collect the broker fee from the tenant.
    pub broker_fee_imposed_on_tenant: bool,
    /// Whether the broker fee is disclosed in the rental listing
    /// and lease agreement per NYC FARE Act § 20-699.21.
    pub other_fees_disclosed_in_listing_and_lease: bool,
    /// Broker fee amount in cents (informational).
    pub broker_fee_amount_cents: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct BrokerFeeAllocationResult {
    pub compliant: bool,
    pub party_responsible: PartyResponsibleForFee,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &BrokerFeeAllocationInput) -> BrokerFeeAllocationResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    match input.regime {
        Regime::NewYorkCityFareAct => {
            let party_responsible = if input.landlord_hired_broker && !input.tenant_hired_own_broker
            {
                notes.push(
                    "NYC FARE Act § 20-699.20 — landlord hired the broker; landlord pays the fee under party-who-hired-pays rule"
                        .to_string(),
                );
                PartyResponsibleForFee::Landlord
            } else if input.tenant_hired_own_broker && !input.landlord_hired_broker {
                notes.push(
                    "NYC FARE Act § 20-699.20 — tenant separately retained their own broker; tenant pays the fee for services they hired"
                        .to_string(),
                );
                PartyResponsibleForFee::Tenant
            } else if input.landlord_hired_broker && input.tenant_hired_own_broker {
                notes.push(
                    "NYC FARE Act — landlord and tenant each hired their own broker; each pays their own broker's fee"
                        .to_string(),
                );
                PartyResponsibleForFee::Landlord
            } else {
                notes.push(
                    "no broker engaged by either party — FARE Act fee allocation does not engage"
                        .to_string(),
                );
                PartyResponsibleForFee::None
            };

            if input.broker_fee_imposed_on_tenant
                && input.landlord_hired_broker
                && !input.tenant_hired_own_broker
            {
                violations.push(
                    "NYC FARE Act § 20-699.20 — landlord or landlord's agent MAY NOT impose or collect broker fee from tenant for services of broker landlord engaged"
                        .to_string(),
                );
            }

            if !input.other_fees_disclosed_in_listing_and_lease {
                violations.push(
                    "NYC FARE Act § 20-699.21 — other rental-related fees must be disclosed in the listing AND the rental agreement"
                        .to_string(),
                );
            }

            notes.push(
                "NYC FARE Act enforced by DCWP (Department of Consumer and Worker Protection) via civil penalty or civil action under § 20-699.22"
                    .to_string(),
            );

            BrokerFeeAllocationResult {
                compliant: violations.is_empty(),
                party_responsible,
                violations,
                citation: citation_for(Regime::NewYorkCityFareAct),
                notes,
            }
        }
        Regime::Default => {
            let party_responsible = if input.tenant_hired_own_broker {
                PartyResponsibleForFee::Tenant
            } else if input.landlord_hired_broker {
                notes.push(
                    "default rule — lease + market practice controls; in tight rental markets landlords traditionally pass broker fees to tenants unless lease specifies otherwise"
                        .to_string(),
                );
                if input.broker_fee_imposed_on_tenant {
                    PartyResponsibleForFee::Tenant
                } else {
                    PartyResponsibleForFee::Landlord
                }
            } else {
                PartyResponsibleForFee::None
            };

            notes.push(
                "Boston Ord. 17-2024 (Boston Municipal Code § 9-6) imposes broker fee disclosure requirements; other municipalities may have local broker fee disclosure rules"
                    .to_string(),
            );

            BrokerFeeAllocationResult {
                compliant: true,
                party_responsible,
                violations,
                citation: citation_for(Regime::Default),
                notes,
            }
        }
    }
}

fn citation_for(regime: Regime) -> &'static str {
    match regime {
        Regime::NewYorkCityFareAct => {
            "NYC Local Law 119 (FARE Act, eff. June 11, 2025); NYC Admin. Code §§ 20-699.20, 20-699.21, 20-699.22"
        }
        Regime::Default => "lease + market practice; Boston Ord. 17-2024 (Boston Municipal Code § 9-6); state broker license statutes",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nyc_base() -> BrokerFeeAllocationInput {
        BrokerFeeAllocationInput {
            regime: Regime::NewYorkCityFareAct,
            landlord_hired_broker: true,
            tenant_hired_own_broker: false,
            broker_fee_imposed_on_tenant: false,
            other_fees_disclosed_in_listing_and_lease: true,
            broker_fee_amount_cents: 350_000,
        }
    }

    fn default_base() -> BrokerFeeAllocationInput {
        BrokerFeeAllocationInput {
            regime: Regime::Default,
            landlord_hired_broker: true,
            tenant_hired_own_broker: false,
            broker_fee_imposed_on_tenant: false,
            other_fees_disclosed_in_listing_and_lease: false,
            broker_fee_amount_cents: 350_000,
        }
    }

    #[test]
    fn nyc_landlord_hired_landlord_pays_compliant() {
        let r = check(&nyc_base());
        assert!(r.compliant);
        assert_eq!(r.party_responsible, PartyResponsibleForFee::Landlord);
    }

    #[test]
    fn nyc_landlord_hired_imposing_fee_on_tenant_violation() {
        let mut i = nyc_base();
        i.broker_fee_imposed_on_tenant = true;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("§ 20-699.20") && v.contains("MAY NOT impose")));
    }

    #[test]
    fn nyc_tenant_hired_own_broker_tenant_pays() {
        let mut i = nyc_base();
        i.landlord_hired_broker = false;
        i.tenant_hired_own_broker = true;
        let r = check(&i);
        assert_eq!(r.party_responsible, PartyResponsibleForFee::Tenant);
        assert!(r.notes.iter().any(|n| n.contains("tenant separately retained")));
    }

    #[test]
    fn nyc_both_parties_hire_their_own_brokers() {
        let mut i = nyc_base();
        i.tenant_hired_own_broker = true;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("each pays their own broker")));
    }

    #[test]
    fn nyc_no_broker_engaged_no_fee_allocation() {
        let mut i = nyc_base();
        i.landlord_hired_broker = false;
        i.tenant_hired_own_broker = false;
        let r = check(&i);
        assert_eq!(r.party_responsible, PartyResponsibleForFee::None);
        assert!(r.notes.iter().any(|n| n.contains("no broker engaged")));
    }

    #[test]
    fn nyc_other_fees_not_disclosed_violation() {
        let mut i = nyc_base();
        i.other_fees_disclosed_in_listing_and_lease = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("§ 20-699.21")));
    }

    #[test]
    fn nyc_dcwp_enforcement_note_always_present() {
        let r = check(&nyc_base());
        assert!(r.notes.iter().any(|n| n.contains("DCWP") && n.contains("§ 20-699.22")));
    }

    #[test]
    fn default_landlord_hired_landlord_pays_by_default() {
        let r = check(&default_base());
        assert!(r.compliant);
        assert_eq!(r.party_responsible, PartyResponsibleForFee::Landlord);
        assert!(r.notes.iter().any(|n| n.contains("lease + market practice")));
    }

    #[test]
    fn default_landlord_hired_passes_to_tenant_via_lease_compliant() {
        let mut i = default_base();
        i.broker_fee_imposed_on_tenant = true;
        let r = check(&i);
        assert!(r.compliant);
        assert_eq!(r.party_responsible, PartyResponsibleForFee::Tenant);
    }

    #[test]
    fn default_tenant_hired_own_broker_tenant_pays() {
        let mut i = default_base();
        i.landlord_hired_broker = false;
        i.tenant_hired_own_broker = true;
        let r = check(&i);
        assert_eq!(r.party_responsible, PartyResponsibleForFee::Tenant);
    }

    #[test]
    fn default_boston_disclosure_note_present() {
        let r = check(&default_base());
        assert!(r.notes.iter().any(|n| n.contains("Boston Ord. 17-2024")));
    }

    #[test]
    fn citation_nyc_fare_act_pins_local_law_119_and_subsections() {
        let r = check(&nyc_base());
        assert!(r.citation.contains("Local Law 119"));
        assert!(r.citation.contains("FARE Act"));
        assert!(r.citation.contains("June 11, 2025"));
        assert!(r.citation.contains("§§ 20-699.20, 20-699.21, 20-699.22"));
    }

    #[test]
    fn citation_default_pins_boston_ord_and_market() {
        let r = check(&default_base());
        assert!(r.citation.contains("lease + market practice"));
        assert!(r.citation.contains("Boston Ord. 17-2024"));
    }

    #[test]
    fn nyc_multiple_violations_accumulate() {
        let mut i = nyc_base();
        i.broker_fee_imposed_on_tenant = true;
        i.other_fees_disclosed_in_listing_and_lease = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert_eq!(r.violations.len(), 2);
    }

    #[test]
    fn nyc_uniquely_prohibits_pass_through_to_tenant_invariant() {
        let mut i_nyc = nyc_base();
        i_nyc.broker_fee_imposed_on_tenant = true;
        let r_nyc = check(&i_nyc);
        assert!(!r_nyc.compliant);

        let mut i_default = default_base();
        i_default.broker_fee_imposed_on_tenant = true;
        let r_default = check(&i_default);
        assert!(r_default.compliant);
    }

    #[test]
    fn nyc_party_who_hired_pays_rule_landlord_hired() {
        let r = check(&nyc_base());
        assert!(r.notes.iter().any(|n| n.contains("party-who-hired-pays rule")));
    }

    #[test]
    fn nyc_tenant_hired_invariant_tenant_pays_in_both_regimes() {
        let mut i_nyc = nyc_base();
        i_nyc.landlord_hired_broker = false;
        i_nyc.tenant_hired_own_broker = true;
        let r_nyc = check(&i_nyc);
        assert_eq!(r_nyc.party_responsible, PartyResponsibleForFee::Tenant);

        let mut i_default = default_base();
        i_default.landlord_hired_broker = false;
        i_default.tenant_hired_own_broker = true;
        let r_default = check(&i_default);
        assert_eq!(r_default.party_responsible, PartyResponsibleForFee::Tenant);
    }

    #[test]
    fn nyc_disclosure_required_even_when_landlord_pays() {
        let mut i = nyc_base();
        i.other_fees_disclosed_in_listing_and_lease = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("§ 20-699.21")));
    }

    #[test]
    fn nyc_full_compliance_path() {
        let r = check(&nyc_base());
        assert!(r.compliant);
        assert!(r.violations.is_empty());
        assert_eq!(r.party_responsible, PartyResponsibleForFee::Landlord);
    }

    #[test]
    fn default_no_disclosure_required_violation() {
        let mut i = default_base();
        i.other_fees_disclosed_in_listing_and_lease = false;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn nyc_landlord_hired_with_tenant_imposition_violates_party_who_hired_rule() {
        let mut i = nyc_base();
        i.broker_fee_imposed_on_tenant = true;
        let r = check(&i);
        assert!(r.violations.iter().any(|v| v.contains("MAY NOT impose")));
    }

    #[test]
    fn default_landlord_hired_no_lease_impose_landlord_pays() {
        let mut i = default_base();
        i.broker_fee_imposed_on_tenant = false;
        let r = check(&i);
        assert_eq!(r.party_responsible, PartyResponsibleForFee::Landlord);
    }

    #[test]
    fn dcwp_enforcement_note_pins_section_20_699_22() {
        let r = check(&nyc_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 20-699.22")));
    }

    #[test]
    fn nyc_civil_penalty_or_action_described_in_enforcement_note() {
        let r = check(&nyc_base());
        assert!(r.notes.iter().any(|n| n.contains("civil penalty") && n.contains("civil action")));
    }

    #[test]
    fn nyc_imposing_fee_on_tenant_but_tenant_self_hired_no_violation() {
        let mut i = nyc_base();
        i.landlord_hired_broker = false;
        i.tenant_hired_own_broker = true;
        i.broker_fee_imposed_on_tenant = true;
        let r = check(&i);
        let imposition_violations: Vec<_> = r.violations.iter().filter(|v| v.contains("MAY NOT impose")).collect();
        assert!(imposition_violations.is_empty(), "tenant who hired own broker may be charged for that broker");
    }

    #[test]
    fn nyc_only_imposes_disclosure_obligation_invariant() {
        let mut i_nyc = nyc_base();
        i_nyc.other_fees_disclosed_in_listing_and_lease = false;
        let r_nyc = check(&i_nyc);
        assert!(!r_nyc.compliant);

        let mut i_default = default_base();
        i_default.other_fees_disclosed_in_listing_and_lease = false;
        let r_default = check(&i_default);
        assert!(r_default.compliant);
    }
}
