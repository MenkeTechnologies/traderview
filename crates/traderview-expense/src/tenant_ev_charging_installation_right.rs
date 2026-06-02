//! Multi-jurisdictional tenant electric vehicle (EV)
//! charging installation right framework. Trader-landlord
//! critical because EV adoption in the U.S. residential
//! market has shifted EVCS-installation requests from
//! novelty to routine — tenant EV ownership rates have
//! climbed steadily and continue to grow. State "right-to-
//! charge" laws now exist in CA + CO + FL + HI + IL + MD +
//! NJ + NY + OR + VA. Failure to grant a qualifying request
//! in a covered jurisdiction triggers breach-of-statute
//! liability and tenant remedies. Companion to
//! tenant_solar_installation, tenant_clothesline_drying_
//! right, rental_satellite_dish_installation_right,
//! rental_broadband_mte_rules.
//!
//! **California Civ. Code § 1947.6** (AB 2565 of 2014,
//! **effective July 1, 2015**) — for any lease executed,
//! extended, or renewed on or after July 1, 2015, a
//! lessor of a dwelling SHALL APPROVE a written request
//! of a lessee to install an EVCS at a parking space
//! allotted for the lessee that meets § 1947.6
//! requirements and complies with the lessor's
//! procedural approval process. Companion: Cal. Civ.
//! Code § 1952.7 (Commercial Leases).
//!
//! **§ 1947.6(d) Exemptions** — § 1947.6 does NOT apply
//! to residential rental properties where:
//! 1. EVCS already exist for lessees at a ratio EQUAL TO
//!    OR GREATER THAN **10 PERCENT** of designated
//!    parking spaces;
//! 2. Parking is NOT provided as part of the lease
//!    agreement;
//! 3. Property has **LESS THAN 5 PARKING SPACES**; OR
//! 4. Dwelling is subject to a public-entity residential
//!    rent control ordinance.
//!
//! **§ 1947.6(c) Tenant obligations** — lessee's written
//! request must include CONSENT TO ENTER WRITTEN
//! AGREEMENT including:
//! 1. Compliance with lessor's installation/use/
//!    maintenance/removal requirements;
//! 2. **Tenant pays for ALL electrical usage** of the
//!    EVCS as part of rent;
//! 3. **Tenant pays for ALL damage, maintenance, repair,
//!    removal, and replacement** of EVCS;
//! 4. **Tenant pays for modifications/improvements** to
//!    property associated with EVCS installation;
//! 5. **§ 1947.6(c)(8) — $1,000,000 LIABILITY INSURANCE**
//!    maintained naming landlord as additional insured.
//!
//! **Colorado HB 23-1233** (effective August 7, 2023) —
//! tenant MAY INSTALL at tenant's expense for tenant's
//! own use a **LEVEL 1 OR LEVEL 2 EVCS** on or in the
//! leased premises. Landlord:
//! 1. SHALL NOT assess or charge any fee for placement
//!    or use of EVCS;
//! 2. MAY require reimbursement for ACTUAL COST of
//!    electricity provided by landlord; OR
//! 3. MAY charge REASONABLE FEE for access (network fee
//!    pass-through permitted).
//!
//! Colorado state Electrical Board must adopt EVCS
//! requirements starting **March 1, 2024**; precluded
//! from adopting rules prohibiting EVCS installation
//! UNLESS rules address a BONA FIDE SAFETY CONCERN.
//!
//! **Maryland HB 830** (Chapter 582 of 2023) — all newly
//! constructed OR renovated housing units with SEPARATE
//! GARAGES, CARPORTS, OR DRIVEWAYS for each unit must
//! include an **EVSE-INSTALLED OR EV-READY parking
//! space**. Right-to-charge applies to existing tenants.
//!
//! **New York General Business Law § 399-zzz +
//! Multiple Dwelling Law amendments** — tenant has right
//! to install EVCS subject to landlord's reasonable
//! procedural approval; landlord may not unreasonably
//! withhold approval.
//!
//! **Default — no statewide right-to-charge framework** —
//! common-law lease modification approval required;
//! landlord may decline tenant request absent compelling
//! reason; ADA reasonable accommodation may apply if
//! tenant has disability-related need.
//!
//! Citations: Cal. Civ. Code § 1947.6 (AB 2565 of 2014,
//! effective July 1, 2015); Cal. Civ. Code § 1952.7
//! (Commercial); Colorado HB 23-1233 (effective August
//! 7, 2023); Maryland HB 830 (Chapter 582 of 2023);
//! Maryland Energy Administration Multifamily Residential
//! EV Study; New York GBL § 399-zzz; states with
//! right-to-charge laws: CA + CO + FL + HI + IL + MD + NJ
//! + NY + OR + VA.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    California,
    Colorado,
    Maryland,
    NewYork,
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TenantEvChargingInstallationRightInput {
    pub jurisdiction: Jurisdiction,
    /// Year lease was executed, extended, or renewed
    /// (CA: ≥ 2015 requires approval under § 1947.6).
    pub lease_year: u32,
    pub lease_month: u32,
    /// Total designated parking spaces at property.
    pub total_parking_spaces: u32,
    /// Existing EVCS count at property (CA 10% exemption).
    pub existing_evcs_count: u32,
    /// Whether parking is provided as part of lease (CA
    /// § 1947.6(d)(2) exemption gate).
    pub parking_provided_in_lease: bool,
    /// Whether property is subject to rent control (CA
    /// § 1947.6(d)(4) exemption gate).
    pub rent_controlled_property: bool,
    /// Whether tenant submitted written request.
    pub written_request_submitted: bool,
    /// Whether tenant consented to written agreement
    /// covering installation/use/maintenance/removal.
    pub written_agreement_consented: bool,
    /// Whether tenant agreed to pay for electrical
    /// usage + damage + maintenance + modifications.
    pub tenant_payment_obligations_accepted: bool,
    /// Whether tenant maintains $1M liability insurance
    /// naming landlord as additional insured (CA
    /// § 1947.6(c)(8)).
    pub one_million_liability_insurance_maintained: bool,
    /// Whether landlord approved the request.
    pub landlord_approved_request: bool,
    /// Whether EVCS is Level 1 or Level 2 (CO HB 23-1233
    /// scope requirement).
    pub level_1_or_2_evcs: bool,
    /// Whether housing is newly constructed or renovated
    /// (MD HB 830 trigger).
    pub newly_constructed_or_renovated: bool,
    /// Whether property has separate garage/carport/
    /// driveway for each unit (MD HB 830 trigger).
    pub separate_garage_carport_driveway_per_unit: bool,
    /// Whether unit has EVSE-installed OR EV-ready
    /// parking space (MD HB 830 compliance).
    pub evse_installed_or_ev_ready_parking_space: bool,
    /// Whether landlord assessed unreasonable fee for
    /// placement/use beyond actual electricity cost +
    /// reasonable access fee (CO HB 23-1233 violation).
    pub unreasonable_fee_assessed: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TenantEvChargingInstallationRightResult {
    pub jurisdiction: Jurisdiction,
    pub right_to_charge_engaged: bool,
    pub approval_obligation_satisfied: bool,
    pub ca_exemption_engaged: bool,
    pub tenant_obligations_compliant: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(
    input: &TenantEvChargingInstallationRightInput,
) -> TenantEvChargingInstallationRightResult {
    let mut failure_reasons: Vec<String> = Vec::new();

    let (right_to_charge_engaged, approval_obligation_satisfied, ca_exemption_engaged, tenant_obligations_compliant) =
        match input.jurisdiction {
            Jurisdiction::California => {
                let post_effective = input.lease_year > 2015
                    || (input.lease_year == 2015 && input.lease_month >= 7);

                let exemption_10pct = input.total_parking_spaces > 0
                    && input.existing_evcs_count * 10 >= input.total_parking_spaces;
                let exemption_no_parking = !input.parking_provided_in_lease;
                let exemption_lt_5_spaces = input.total_parking_spaces < 5;
                let exemption_rent_controlled = input.rent_controlled_property;
                let exemption_engaged = exemption_10pct
                    || exemption_no_parking
                    || exemption_lt_5_spaces
                    || exemption_rent_controlled;

                let right_engaged = post_effective && !exemption_engaged
                    && input.written_request_submitted;

                let tenant_obligations_complete = input.written_agreement_consented
                    && input.tenant_payment_obligations_accepted
                    && input.one_million_liability_insurance_maintained;

                let approval_satisfied = !right_engaged || input.landlord_approved_request;

                if right_engaged && !tenant_obligations_complete {
                    if !input.written_agreement_consented {
                        failure_reasons.push(
                            "Cal. Civ. Code § 1947.6(c) — lessee's written request must include CONSENT TO ENTER written agreement covering compliance with lessor's installation/use/maintenance/removal requirements".to_string(),
                        );
                    }
                    if !input.tenant_payment_obligations_accepted {
                        failure_reasons.push(
                            "Cal. Civ. Code § 1947.6(c)(1)-(7) — tenant must agree to pay for ALL electrical usage + damage + maintenance + repair + removal + replacement + modifications/improvements to property associated with EVCS installation".to_string(),
                        );
                    }
                    if !input.one_million_liability_insurance_maintained {
                        failure_reasons.push(
                            "Cal. Civ. Code § 1947.6(c)(8) — tenant MUST maintain $1,000,000 LIABILITY INSURANCE naming landlord as ADDITIONAL INSURED".to_string(),
                        );
                    }
                }

                if right_engaged && !input.landlord_approved_request {
                    failure_reasons.push(
                        "Cal. Civ. Code § 1947.6(a) — for any lease executed, extended, or renewed on or after July 1, 2015, lessor SHALL APPROVE written tenant request to install EVCS at allotted parking space that meets § 1947.6 requirements and complies with lessor's procedural approval process".to_string(),
                    );
                }

                (right_engaged, approval_satisfied, exemption_engaged, tenant_obligations_complete)
            }
            Jurisdiction::Colorado => {
                let post_effective = input.lease_year > 2023
                    || (input.lease_year == 2023 && input.lease_month >= 8);
                let right_engaged = post_effective && input.level_1_or_2_evcs;
                let approval_satisfied = !right_engaged || (!input.unreasonable_fee_assessed && input.landlord_approved_request);

                if right_engaged && input.unreasonable_fee_assessed {
                    failure_reasons.push(
                        "Colorado HB 23-1233 (effective August 7, 2023) — landlord SHALL NOT assess or charge any fee for placement or use of EVCS; landlord MAY require reimbursement for ACTUAL COST of electricity provided OR REASONABLE access fee (with network fee pass-through); unreasonable fee imposition violates statute".to_string(),
                    );
                }

                if right_engaged && !input.landlord_approved_request {
                    failure_reasons.push(
                        "Colorado HB 23-1233 — tenant MAY install at tenant's expense for own use Level 1 OR Level 2 EVCS on or in leased premises; landlord refusal absent bona fide safety concern violates right-to-charge".to_string(),
                    );
                }

                (right_engaged, approval_satisfied, false, true)
            }
            Jurisdiction::Maryland => {
                let post_effective = input.lease_year >= 2023;
                let right_engaged = post_effective
                    && input.newly_constructed_or_renovated
                    && input.separate_garage_carport_driveway_per_unit;
                let approval_satisfied = !right_engaged || input.evse_installed_or_ev_ready_parking_space;

                if right_engaged && !input.evse_installed_or_ev_ready_parking_space {
                    failure_reasons.push(
                        "Maryland HB 830 (Chapter 582 of 2023) — all newly constructed OR renovated housing units with SEPARATE GARAGES, CARPORTS, OR DRIVEWAYS for each unit MUST include an EVSE-INSTALLED OR EV-READY parking space".to_string(),
                    );
                }

                (right_engaged, approval_satisfied, false, true)
            }
            Jurisdiction::NewYork => {
                let right_engaged = input.written_request_submitted;
                let approval_satisfied = !right_engaged || input.landlord_approved_request;

                if right_engaged && !input.landlord_approved_request {
                    failure_reasons.push(
                        "NY Gen. Bus. Law § 399-zzz + NY Multiple Dwelling Law amendments — tenant has right to install EVCS subject to landlord's reasonable procedural approval; landlord MAY NOT unreasonably withhold approval".to_string(),
                    );
                }

                (right_engaged, approval_satisfied, false, true)
            }
            Jurisdiction::Default => {
                let right_engaged = false;
                let approval_satisfied = true;
                (right_engaged, approval_satisfied, false, true)
            }
        };

    let notes: Vec<String> = vec![
        "Cal. Civ. Code § 1947.6(a) (AB 2565 of 2014, effective July 1, 2015) — for any lease executed, extended, or renewed on or after July 1, 2015, lessor of a dwelling SHALL APPROVE written tenant request to install EVCS at parking space allotted for lessee that meets § 1947.6 requirements".to_string(),
        "Cal. Civ. Code § 1947.6(d) four exemptions — § 1947.6 does NOT apply where (1) EVCS exist at 10%+ of designated parking spaces; (2) parking NOT provided as part of lease; (3) less than 5 parking spaces; (4) dwelling subject to rent control ordinance".to_string(),
        "Cal. Civ. Code § 1947.6(c)(1)-(7) tenant obligations — written request must include consent to written agreement covering: (1) compliance with installation/use/maintenance/removal requirements; (2) payment of ALL electrical usage as part of rent; (3) payment of ALL damage, maintenance, repair, removal, replacement; (4) payment of modifications/improvements to property".to_string(),
        "Cal. Civ. Code § 1947.6(c)(8) — tenant MUST maintain $1,000,000 LIABILITY INSURANCE naming landlord as ADDITIONAL INSURED".to_string(),
        "Cal. Civ. Code § 1952.7 — commercial-lease companion provision; parallel structure to § 1947.6 residential framework".to_string(),
        "Colorado HB 23-1233 (effective August 7, 2023) — tenant MAY INSTALL at tenant's expense for own use Level 1 OR Level 2 EVCS on or in leased premises; landlord SHALL NOT assess fee for placement/use; MAY require reimbursement for actual electricity cost OR reasonable access fee with network fee pass-through".to_string(),
        "Colorado HB 23-1233 — state Electrical Board must adopt EVCS requirements starting March 1, 2024; precluded from adopting rules prohibiting EVCS UNLESS rules address BONA FIDE SAFETY CONCERN".to_string(),
        "Maryland HB 830 (Chapter 582 of 2023) — all newly constructed OR renovated housing units with separate garages, carports, or driveways for each unit MUST include EVSE-INSTALLED OR EV-READY parking space; Maryland Energy Administration Multifamily Residential EV Study implementation guidance".to_string(),
        "NY Gen. Bus. Law § 399-zzz + NY Multiple Dwelling Law amendments — tenant has right to install EVCS subject to landlord's reasonable procedural approval; landlord MAY NOT unreasonably withhold approval".to_string(),
        "Default — no statewide right-to-charge framework; common-law lease modification approval required; landlord may decline absent compelling reason; ADA reasonable accommodation may apply if tenant has disability-related need".to_string(),
        "States with right-to-charge laws currently in effect: California + Colorado + Florida + Hawaii + Illinois + Maryland + New Jersey + New York + Oregon + Virginia".to_string(),
        "Cross-jurisdictional architecture: California uses APPROVAL-MANDATE with FOUR EXEMPTIONS + tenant insurance/payment obligations; Colorado uses PROHIBITION-OF-FEE-PLUS-ACTUAL-COST-PASS-THROUGH; Maryland uses NEW-CONSTRUCTION-INFRASTRUCTURE-MANDATE; NY uses REASONABLENESS-REVIEW; Default common-law".to_string(),
    ];

    TenantEvChargingInstallationRightResult {
        jurisdiction: input.jurisdiction,
        right_to_charge_engaged,
        approval_obligation_satisfied,
        ca_exemption_engaged,
        tenant_obligations_compliant,
        failure_reasons,
        citation: "Cal. Civ. Code § 1947.6 (AB 2565 of 2014, effective July 1, 2015); Cal. Civ. Code § 1952.7 (Commercial); Colorado HB 23-1233 (effective August 7, 2023); Maryland HB 830 (Chapter 582 of 2023); Maryland Energy Administration Multifamily Residential EV Study; NY Gen. Bus. Law § 399-zzz",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ca_compliant() -> TenantEvChargingInstallationRightInput {
        TenantEvChargingInstallationRightInput {
            jurisdiction: Jurisdiction::California,
            lease_year: 2026,
            lease_month: 6,
            total_parking_spaces: 20,
            existing_evcs_count: 1,
            parking_provided_in_lease: true,
            rent_controlled_property: false,
            written_request_submitted: true,
            written_agreement_consented: true,
            tenant_payment_obligations_accepted: true,
            one_million_liability_insurance_maintained: true,
            landlord_approved_request: true,
            level_1_or_2_evcs: true,
            newly_constructed_or_renovated: false,
            separate_garage_carport_driveway_per_unit: false,
            evse_installed_or_ev_ready_parking_space: false,
            unreasonable_fee_assessed: false,
        }
    }

    #[test]
    fn ca_post_july_2015_full_compliance_engaged() {
        let r = check(&ca_compliant());
        assert!(r.right_to_charge_engaged);
        assert!(r.approval_obligation_satisfied);
        assert!(r.tenant_obligations_compliant);
    }

    #[test]
    fn ca_pre_july_2015_no_obligation() {
        let mut i = ca_compliant();
        i.lease_year = 2015;
        i.lease_month = 6;
        let r = check(&i);
        assert!(!r.right_to_charge_engaged);
    }

    #[test]
    fn ca_july_2015_boundary_engages() {
        let mut i = ca_compliant();
        i.lease_year = 2015;
        i.lease_month = 7;
        let r = check(&i);
        assert!(r.right_to_charge_engaged);
    }

    #[test]
    fn ca_exemption_10_percent_evcs_exist() {
        let mut i = ca_compliant();
        i.total_parking_spaces = 20;
        i.existing_evcs_count = 2;
        let r = check(&i);
        assert!(r.ca_exemption_engaged);
        assert!(!r.right_to_charge_engaged);
    }

    #[test]
    fn ca_exemption_no_parking_in_lease() {
        let mut i = ca_compliant();
        i.parking_provided_in_lease = false;
        let r = check(&i);
        assert!(r.ca_exemption_engaged);
    }

    #[test]
    fn ca_exemption_under_5_parking_spaces() {
        let mut i = ca_compliant();
        i.total_parking_spaces = 4;
        let r = check(&i);
        assert!(r.ca_exemption_engaged);
    }

    #[test]
    fn ca_exemption_5_parking_spaces_no_exemption() {
        let mut i = ca_compliant();
        i.total_parking_spaces = 5;
        i.existing_evcs_count = 0;
        let r = check(&i);
        assert!(!r.ca_exemption_engaged);
    }

    #[test]
    fn ca_exemption_rent_controlled() {
        let mut i = ca_compliant();
        i.rent_controlled_property = true;
        let r = check(&i);
        assert!(r.ca_exemption_engaged);
    }

    #[test]
    fn ca_no_written_request_no_engagement() {
        let mut i = ca_compliant();
        i.written_request_submitted = false;
        let r = check(&i);
        assert!(!r.right_to_charge_engaged);
    }

    #[test]
    fn ca_missing_written_agreement_violation() {
        let mut i = ca_compliant();
        i.written_agreement_consented = false;
        let r = check(&i);
        assert!(!r.tenant_obligations_compliant);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 1947.6(c)")
            && f.contains("CONSENT TO ENTER")));
    }

    #[test]
    fn ca_missing_payment_obligations_violation() {
        let mut i = ca_compliant();
        i.tenant_payment_obligations_accepted = false;
        let r = check(&i);
        assert!(!r.tenant_obligations_compliant);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 1947.6(c)(1)-(7)")
            && f.contains("electrical usage")
            && f.contains("damage")));
    }

    #[test]
    fn ca_missing_1m_insurance_violation() {
        let mut i = ca_compliant();
        i.one_million_liability_insurance_maintained = false;
        let r = check(&i);
        assert!(!r.tenant_obligations_compliant);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 1947.6(c)(8)")
            && f.contains("$1,000,000 LIABILITY INSURANCE")
            && f.contains("ADDITIONAL INSURED")));
    }

    #[test]
    fn ca_landlord_refusal_when_obligated_violation() {
        let mut i = ca_compliant();
        i.landlord_approved_request = false;
        let r = check(&i);
        assert!(!r.approval_obligation_satisfied);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 1947.6(a)")
            && f.contains("July 1, 2015")
            && f.contains("SHALL APPROVE")));
    }

    #[test]
    fn colorado_post_august_2023_engages() {
        let mut i = ca_compliant();
        i.jurisdiction = Jurisdiction::Colorado;
        i.lease_year = 2024;
        i.level_1_or_2_evcs = true;
        let r = check(&i);
        assert!(r.right_to_charge_engaged);
    }

    #[test]
    fn colorado_pre_august_2023_no_engagement() {
        let mut i = ca_compliant();
        i.jurisdiction = Jurisdiction::Colorado;
        i.lease_year = 2023;
        i.lease_month = 7;
        i.level_1_or_2_evcs = true;
        let r = check(&i);
        assert!(!r.right_to_charge_engaged);
    }

    #[test]
    fn colorado_unreasonable_fee_violation() {
        let mut i = ca_compliant();
        i.jurisdiction = Jurisdiction::Colorado;
        i.lease_year = 2024;
        i.unreasonable_fee_assessed = true;
        let r = check(&i);
        assert!(!r.approval_obligation_satisfied);
        assert!(r.failure_reasons.iter().any(|f| f.contains("HB 23-1233")
            && f.contains("SHALL NOT assess")));
    }

    #[test]
    fn maryland_new_construction_no_evse_installed_violation() {
        let mut i = ca_compliant();
        i.jurisdiction = Jurisdiction::Maryland;
        i.lease_year = 2024;
        i.newly_constructed_or_renovated = true;
        i.separate_garage_carport_driveway_per_unit = true;
        i.evse_installed_or_ev_ready_parking_space = false;
        let r = check(&i);
        assert!(r.right_to_charge_engaged);
        assert!(!r.approval_obligation_satisfied);
        assert!(r.failure_reasons.iter().any(|f| f.contains("HB 830")
            && f.contains("Chapter 582 of 2023")
            && f.contains("EVSE-INSTALLED OR EV-READY")));
    }

    #[test]
    fn maryland_new_construction_with_evse_compliant() {
        let mut i = ca_compliant();
        i.jurisdiction = Jurisdiction::Maryland;
        i.lease_year = 2024;
        i.newly_constructed_or_renovated = true;
        i.separate_garage_carport_driveway_per_unit = true;
        i.evse_installed_or_ev_ready_parking_space = true;
        let r = check(&i);
        assert!(r.approval_obligation_satisfied);
    }

    #[test]
    fn maryland_existing_construction_no_obligation() {
        let mut i = ca_compliant();
        i.jurisdiction = Jurisdiction::Maryland;
        i.lease_year = 2024;
        i.newly_constructed_or_renovated = false;
        let r = check(&i);
        assert!(!r.right_to_charge_engaged);
    }

    #[test]
    fn ny_written_request_engages_obligation() {
        let mut i = ca_compliant();
        i.jurisdiction = Jurisdiction::NewYork;
        let r = check(&i);
        assert!(r.right_to_charge_engaged);
    }

    #[test]
    fn ny_unreasonable_withholding_violation() {
        let mut i = ca_compliant();
        i.jurisdiction = Jurisdiction::NewYork;
        i.landlord_approved_request = false;
        let r = check(&i);
        assert!(!r.approval_obligation_satisfied);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 399-zzz")
            && f.contains("unreasonably withhold")));
    }

    #[test]
    fn default_no_obligation() {
        let mut i = ca_compliant();
        i.jurisdiction = Jurisdiction::Default;
        let r = check(&i);
        assert!(!r.right_to_charge_engaged);
        assert!(r.approval_obligation_satisfied);
    }

    #[test]
    fn jurisdiction_truth_table_five_cells() {
        for jur in [
            Jurisdiction::California,
            Jurisdiction::Colorado,
            Jurisdiction::Maryland,
            Jurisdiction::NewYork,
            Jurisdiction::Default,
        ] {
            let mut i = ca_compliant();
            i.jurisdiction = jur;
            let r = check(&i);
            assert_eq!(r.jurisdiction, jur);
        }
    }

    #[test]
    fn ca_uniquely_provides_four_exemptions_invariant() {
        let mut ca = ca_compliant();
        ca.total_parking_spaces = 3;
        let r_ca = check(&ca);
        assert!(r_ca.ca_exemption_engaged);

        for jur in [
            Jurisdiction::Colorado,
            Jurisdiction::Maryland,
            Jurisdiction::NewYork,
            Jurisdiction::Default,
        ] {
            let mut i = ca_compliant();
            i.jurisdiction = jur;
            i.total_parking_spaces = 3;
            let r = check(&i);
            assert!(!r.ca_exemption_engaged, "jur={:?}", jur);
        }
    }

    #[test]
    fn citation_pins_all_jurisdictions() {
        let r = check(&ca_compliant());
        assert!(r.citation.contains("Cal. Civ. Code § 1947.6"));
        assert!(r.citation.contains("AB 2565 of 2014"));
        assert!(r.citation.contains("July 1, 2015"));
        assert!(r.citation.contains("Cal. Civ. Code § 1952.7"));
        assert!(r.citation.contains("Colorado HB 23-1233"));
        assert!(r.citation.contains("August 7, 2023"));
        assert!(r.citation.contains("Maryland HB 830"));
        assert!(r.citation.contains("Chapter 582 of 2023"));
        assert!(r.citation.contains("NY Gen. Bus. Law § 399-zzz"));
    }

    #[test]
    fn note_pins_ca_approval_mandate() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 1947.6(a)")
            && n.contains("AB 2565 of 2014")
            && n.contains("July 1, 2015")
            && n.contains("SHALL APPROVE")));
    }

    #[test]
    fn note_pins_ca_four_exemptions() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 1947.6(d)")
            && n.contains("10%+")
            && n.contains("less than 5 parking spaces")
            && n.contains("rent control")));
    }

    #[test]
    fn note_pins_ca_tenant_payment_obligations() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 1947.6(c)(1)-(7)")
            && n.contains("electrical usage")
            && n.contains("damage")
            && n.contains("modifications/improvements")));
    }

    #[test]
    fn note_pins_ca_1m_insurance_requirement() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 1947.6(c)(8)")
            && n.contains("$1,000,000 LIABILITY INSURANCE")
            && n.contains("ADDITIONAL INSURED")));
    }

    #[test]
    fn note_pins_ca_commercial_companion() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 1952.7")
            && n.contains("commercial-lease companion")));
    }

    #[test]
    fn note_pins_co_hb_23_1233_no_fee_actual_cost() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("Colorado HB 23-1233")
            && n.contains("August 7, 2023")
            && n.contains("Level 1 OR Level 2")
            && n.contains("network fee pass-through")));
    }

    #[test]
    fn note_pins_co_electrical_board_2024() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("Colorado HB 23-1233")
            && n.contains("March 1, 2024")
            && n.contains("BONA FIDE SAFETY CONCERN")));
    }

    #[test]
    fn note_pins_md_hb_830_new_construction() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("Maryland HB 830")
            && n.contains("Chapter 582 of 2023")
            && n.contains("EVSE-INSTALLED OR EV-READY")));
    }

    #[test]
    fn note_pins_ny_399_zzz_reasonable_review() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 399-zzz")
            && n.contains("reasonable procedural approval")
            && n.contains("unreasonably withhold")));
    }

    #[test]
    fn note_pins_default_common_law_ada() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("Default")
            && n.contains("ADA reasonable accommodation")));
    }

    #[test]
    fn note_pins_ten_state_right_to_charge_list() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("States with right-to-charge laws")
            && n.contains("California")
            && n.contains("Colorado")
            && n.contains("Florida")
            && n.contains("Hawaii")
            && n.contains("Illinois")
            && n.contains("Maryland")));
    }

    #[test]
    fn note_pins_cross_jurisdictional_architecture() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("Cross-jurisdictional architecture")
            && n.contains("APPROVAL-MANDATE")
            && n.contains("PROHIBITION-OF-FEE")
            && n.contains("NEW-CONSTRUCTION-INFRASTRUCTURE-MANDATE")
            && n.contains("REASONABLENESS-REVIEW")));
    }

    #[test]
    fn multiple_ca_failures_stack() {
        let mut i = ca_compliant();
        i.written_agreement_consented = false;
        i.tenant_payment_obligations_accepted = false;
        i.one_million_liability_insurance_maintained = false;
        i.landlord_approved_request = false;
        let r = check(&i);
        assert!(r.failure_reasons.len() >= 4);
    }
}
