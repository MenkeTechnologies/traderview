//! Multi-jurisdictional residential rental flood hazard
//! disclosure framework — climate-era statutory disclosure
//! regime added across multiple states between 2018 and
//! 2024. Trader-landlord critical because waterfront /
//! coastal / floodplain-zone investment properties (very
//! common in trader real-estate portfolios) trigger
//! mandatory written pre-lease disclosure under state law,
//! and failure routes directly to tenant LEASE
//! TERMINATION right plus damages exposure. Companion to
//! rental_basement_water_intrusion_disclosure (subsurface
//! water), rental_sinkhole_disclosure (FL specific),
//! rental_property_registration (registration regimes),
//! tenant_in_foreclosure_protection.
//!
//! **California Gov. Code § 8589.45** (AB 646 of 2017,
//! effective **July 1, 2018**) — every residential lease
//! or rental agreement entered into on or after July 1,
//! 2018 must include written flood-hazard disclosure when
//! owner has ACTUAL KNOWLEDGE that the property is in:
//! 1. A FEMA-designated **Special Flood Hazard Area**
//!    (100-year floodplain); OR
//! 2. An **area of potential flooding** identified by Cal
//!    OES.
//!
//! Required content:
//! - Statement that property is in a special flood hazard
//!   area or area of potential flooding (if owner has
//!   actual knowledge).
//! - Statement that tenant may obtain hazard information
//!   from Cal OES **MyHazards** website (URL required in
//!   disclosure).
//! - Statement that owner's insurance does NOT cover loss
//!   of tenant's personal possessions; tenant should
//!   consider purchasing renter's + flood insurance.
//! - Statement that owner is not required to provide
//!   additional information and information provided is
//!   deemed adequate.
//!
//! Format: **8-point minimum type** size.
//!
//! **Texas Prop. Code § 92.0135** (HB 531 of 2021,
//! effective **January 1, 2022**) — landlord must provide
//! written flood notice to tenant before lease signing
//! covering:
//! 1. Whether landlord is aware that dwelling is located
//!    in **100-year floodplain** (FEMA Special Flood
//!    Hazard Area). NOT required if elevation of dwelling
//!    is raised above 100-year floodplain per federal
//!    regulations.
//! 2. Whether landlord knows that flooding damaged any
//!    portion of dwelling at least once during the
//!    **5-year period immediately preceding** lease
//!    effective date.
//!
//! Tenant remedy for failure: if tenant suffers
//! **SUBSTANTIAL LOSS or damage to personal property as a
//! result of flooding** (50%+ of value of repairs/
//! replacement), tenant may **TERMINATE LEASE within 30
//! days of the loss**.
//!
//! **New Jersey Flood Risk Notification Law** (effective
//! **March 20, 2024**) — N.J.S.A. 46:8-50 et seq. —
//! landlords of:
//! 1. Commercial space; OR
//! 2. Residential dwellings in premises containing **MORE
//!    THAN TWO** units (or more than three units where one
//!    is owner-occupied)
//!
//! MUST notify each tenant in writing PRIOR TO lease
//! signing or renewal whether the property is in:
//! 1. A FEMA **Special Flood Hazard Area** (100-year
//!    floodplain); OR
//! 2. A **Moderate Risk Flood Hazard Area** (500-year
//!    floodplain).
//!
//! Format: separate rider, **12-point minimum typeface**,
//! **signed or acknowledged** by tenant.
//!
//! Tenant remedy for failure: **right to TERMINATE LEASE
//! for which landlord has failed to make required flood
//! disclosures** plus statutory damages and attorney fees
//! per N.J.S.A. 46:8-50(d).
//!
//! **Default — no statewide flood disclosure regime** —
//! common-law fraudulent concealment requires (1) actual
//! or constructive knowledge of flood-hazard material
//! defect; (2) concealment; (3) tenant reasonable
//! reliance; (4) damages. Most states also have implied
//! warranty of habitability that may reach flood-related
//! water intrusion. **FEMA NFIP flood insurance** is
//! separately available regardless of state disclosure
//! regime.
//!
//! Citations: Cal. Gov. Code § 8589.45 (AB 646 of 2017);
//! Tex. Prop. Code § 92.0135 (HB 531 of 2021); N.J.S.A.
//! 46:8-50 et seq. (NJ Flood Risk Notification Law 2024);
//! FEMA Special Flood Hazard Area definitions 44 CFR
//! § 59.1 and § 60.3.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    California,
    Texas,
    NewJersey,
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FloodHazardDisclosureInput {
    pub jurisdiction: Jurisdiction,
    pub lease_year: u32,
    pub lease_month: u32,
    /// Whether property is in FEMA Special Flood Hazard
    /// Area (100-year floodplain).
    pub in_special_flood_hazard_area: bool,
    /// Whether property is in Moderate Risk Flood Hazard
    /// Area (500-year floodplain).
    pub in_moderate_risk_flood_area: bool,
    /// Whether landlord has actual knowledge of flood
    /// hazard (CA element).
    pub landlord_has_actual_knowledge: bool,
    /// Whether dwelling elevation is raised above 100-year
    /// floodplain per federal regulations (TX exception).
    pub elevation_above_floodplain_federal_compliant: bool,
    /// Whether landlord knows of flooding damage to any
    /// portion of dwelling in 5 years preceding lease
    /// effective date (TX prong 2).
    pub landlord_knows_5_year_prior_flooding: bool,
    /// Number of dwelling units in property (NJ 3+ unit
    /// threshold OR 4+ unit if owner-occupied threshold).
    pub dwelling_unit_count: u32,
    /// Whether owner occupies one of the units (NJ
    /// threshold modifier).
    pub owner_occupied: bool,
    /// Whether commercial space (NJ commercial trigger).
    pub commercial_property: bool,
    /// Whether written disclosure was provided before
    /// lease signing.
    pub disclosure_provided_before_lease: bool,
    /// Whether disclosure included MyHazards URL (CA
    /// content element).
    pub myhazards_url_included: bool,
    /// Whether disclosure included renter's + flood
    /// insurance recommendation (CA content element).
    pub insurance_recommendation_included: bool,
    /// Whether disclosure was in CA-required 8-point min
    /// type OR NJ-required 12-point min type.
    pub minimum_type_size_satisfied: bool,
    /// Whether NJ-required separate rider with tenant
    /// signature/acknowledgement (NJ format element).
    pub nj_separate_rider_with_signature: bool,
    /// Whether tenant suffered substantial loss (50%+ of
    /// personal property repair/replacement value) (TX
    /// remedy gate).
    pub tenant_substantial_loss_engaged: bool,
    /// Days since tenant's substantial loss occurred (TX
    /// 30-day termination clock).
    pub days_since_substantial_loss: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FloodHazardDisclosureResult {
    pub jurisdiction: Jurisdiction,
    pub disclosure_obligation_triggered: bool,
    pub disclosure_compliant: bool,
    pub tenant_lease_termination_right_engaged: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &FloodHazardDisclosureInput) -> FloodHazardDisclosureResult {
    let mut failure_reasons: Vec<String> = Vec::new();

    let (disclosure_obligation_triggered, disclosure_compliant, tenant_lease_termination_right_engaged) =
        match input.jurisdiction {
            Jurisdiction::California => {
                let post_effective = input.lease_year > 2018
                    || (input.lease_year == 2018 && input.lease_month >= 7);
                let obligation_triggered = post_effective
                    && input.landlord_has_actual_knowledge
                    && input.in_special_flood_hazard_area;

                let mut compliant = true;
                if obligation_triggered {
                    if !input.disclosure_provided_before_lease {
                        compliant = false;
                        failure_reasons.push(
                            "Cal. Gov. Code § 8589.45(a) — every residential lease entered into on or after July 1, 2018 MUST include written flood-hazard disclosure when owner has ACTUAL KNOWLEDGE that the property is in a special flood hazard area or area of potential flooding".to_string(),
                        );
                    }
                    if !input.myhazards_url_included {
                        compliant = false;
                        failure_reasons.push(
                            "Cal. Gov. Code § 8589.45(a)(2) — disclosure MUST state that tenant may obtain hazard information from Cal OES MyHazards website (URL required in disclosure)".to_string(),
                        );
                    }
                    if !input.insurance_recommendation_included {
                        compliant = false;
                        failure_reasons.push(
                            "Cal. Gov. Code § 8589.45(a)(3) — disclosure MUST state that owner's insurance does NOT cover loss of tenant's personal possessions and recommend tenant purchase renter's insurance and flood insurance".to_string(),
                        );
                    }
                    if !input.minimum_type_size_satisfied {
                        compliant = false;
                        failure_reasons.push(
                            "Cal. Gov. Code § 8589.45(c) — disclosure MUST be in 8-point minimum type size".to_string(),
                        );
                    }
                }
                (obligation_triggered, compliant, false)
            }
            Jurisdiction::Texas => {
                let post_effective = input.lease_year >= 2022;
                let prong_1_triggered = input.in_special_flood_hazard_area
                    && !input.elevation_above_floodplain_federal_compliant;
                let prong_2_triggered = input.landlord_knows_5_year_prior_flooding;
                let obligation_triggered =
                    post_effective && (prong_1_triggered || prong_2_triggered);

                let compliant = !obligation_triggered || input.disclosure_provided_before_lease;
                if obligation_triggered && !compliant {
                    failure_reasons.push(
                        "Tex. Prop. Code § 92.0135 — landlord must provide written flood notice before lease signing: (1) whether dwelling located in 100-year floodplain (unless elevation raised above per federal regulations); (2) whether flooding damaged any portion of dwelling within 5 years preceding lease".to_string(),
                    );
                }

                let tx_terminate = obligation_triggered
                    && !compliant
                    && input.tenant_substantial_loss_engaged
                    && input.days_since_substantial_loss <= 30;
                if obligation_triggered
                    && !compliant
                    && input.tenant_substantial_loss_engaged
                {
                    failure_reasons.push(format!(
                        "Tex. Prop. Code § 92.0135(c) — tenant remedy: if substantial loss (50%+ of personal property value) results from flooding AND landlord failed to disclose, tenant may TERMINATE LEASE within 30 days of loss; current {} days post-loss",
                        input.days_since_substantial_loss
                    ));
                }
                (obligation_triggered, compliant, tx_terminate)
            }
            Jurisdiction::NewJersey => {
                let post_effective = input.lease_year > 2024
                    || (input.lease_year == 2024 && input.lease_month >= 3);
                let unit_threshold = if input.owner_occupied {
                    input.dwelling_unit_count > 3
                } else {
                    input.dwelling_unit_count > 2
                };
                let obligation_triggered = post_effective
                    && (input.commercial_property || unit_threshold)
                    && (input.in_special_flood_hazard_area || input.in_moderate_risk_flood_area);

                let mut compliant = true;
                if obligation_triggered {
                    if !input.disclosure_provided_before_lease {
                        compliant = false;
                        failure_reasons.push(
                            "N.J.S.A. 46:8-50 (NJ Flood Risk Notification Law, effective March 20, 2024) — landlords of commercial space OR residential premises containing MORE THAN 2 dwelling units (or more than 3 where one is owner-occupied) MUST notify tenant in writing PRIOR TO lease signing/renewal of FEMA Special Flood Hazard Area (100-year) OR Moderate Risk Flood Hazard Area (500-year)".to_string(),
                        );
                    }
                    if !input.nj_separate_rider_with_signature {
                        compliant = false;
                        failure_reasons.push(
                            "N.J.S.A. 46:8-50(c) — for residential leases, written notice must appear as SEPARATE RIDER, 12-point minimum typeface, SIGNED OR ACKNOWLEDGED by tenant".to_string(),
                        );
                    }
                    if !input.minimum_type_size_satisfied {
                        compliant = false;
                        failure_reasons.push(
                            "N.J.S.A. 46:8-50(c) — disclosure rider must be in 12-point MINIMUM typeface".to_string(),
                        );
                    }
                }

                let nj_terminate = obligation_triggered && !compliant;
                if nj_terminate {
                    failure_reasons.push(
                        "N.J.S.A. 46:8-50(d) — tenant remedy: right to TERMINATE LEASE for which landlord has failed to make required flood disclosures + statutory damages + attorney fees".to_string(),
                    );
                }
                (obligation_triggered, compliant, nj_terminate)
            }
            Jurisdiction::Default => {
                let obligation_triggered = false;
                let compliant = true;
                (obligation_triggered, compliant, false)
            }
        };

    let notes: Vec<String> = vec![
        "Cal. Gov. Code § 8589.45 (AB 646 of 2017, effective July 1, 2018) — every residential lease entered into on or after July 1, 2018 must include written flood-hazard disclosure when owner has ACTUAL KNOWLEDGE that property is in FEMA Special Flood Hazard Area OR Cal OES area of potential flooding".to_string(),
        "Cal. Gov. Code § 8589.45(a)(2) — disclosure must include reference to Cal OES MyHazards website URL where tenant may obtain hazard information".to_string(),
        "Cal. Gov. Code § 8589.45(a)(3) — disclosure must state that owner's insurance does NOT cover tenant personal property and recommend purchase of renter's AND flood insurance".to_string(),
        "Cal. Gov. Code § 8589.45(c) — disclosure must be in 8-point MINIMUM type size".to_string(),
        "Tex. Prop. Code § 92.0135 (HB 531 of 2021, effective January 1, 2022) — landlord must provide written flood notice before lease signing covering (1) 100-year floodplain status (unless elevation above per federal regulations) AND (2) 5-year prior flooding damage knowledge".to_string(),
        "Tex. Prop. Code § 92.0135(c) — tenant remedy: substantial loss (50%+ of personal property repair/replacement value) from flooding plus failed disclosure permits TENANT TO TERMINATE LEASE within 30 DAYS OF LOSS".to_string(),
        "N.J.S.A. 46:8-50 et seq. (NJ Flood Risk Notification Law, effective March 20, 2024) — landlords of commercial space OR residential premises with MORE THAN 2 units (or more than 3 owner-occupied) MUST disclose FEMA 100-year Special Flood Hazard Area OR 500-year Moderate Risk Flood Hazard Area".to_string(),
        "N.J.S.A. 46:8-50(c) — disclosure must be SEPARATE RIDER, 12-point MINIMUM typeface, SIGNED OR ACKNOWLEDGED by tenant".to_string(),
        "N.J.S.A. 46:8-50(d) — tenant remedy: right to TERMINATE LEASE plus statutory damages plus attorney fees".to_string(),
        "Default — no statewide flood disclosure regime: common-law fraudulent concealment requires (1) actual/constructive knowledge; (2) concealment; (3) reasonable reliance; (4) damages; plus implied warranty of habitability may reach flood-related water intrusion".to_string(),
        "FEMA NFIP flood insurance — separately available regardless of state disclosure regime; 44 CFR § 59.1 + § 60.3 governs Special Flood Hazard Area definitions and zone designations (A + AE + AH + AO + AR + V + VE)".to_string(),
    ];

    FloodHazardDisclosureResult {
        jurisdiction: input.jurisdiction,
        disclosure_obligation_triggered,
        disclosure_compliant,
        tenant_lease_termination_right_engaged,
        failure_reasons,
        citation: "Cal. Gov. Code § 8589.45 (AB 646 of 2017); Tex. Prop. Code § 92.0135 (HB 531 of 2021); N.J.S.A. 46:8-50 et seq. (NJ Flood Risk Notification Law of 2024); FEMA NFIP regulations 44 CFR § 59.1 and § 60.3",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ca_baseline() -> FloodHazardDisclosureInput {
        FloodHazardDisclosureInput {
            jurisdiction: Jurisdiction::California,
            lease_year: 2026,
            lease_month: 6,
            in_special_flood_hazard_area: true,
            in_moderate_risk_flood_area: false,
            landlord_has_actual_knowledge: true,
            elevation_above_floodplain_federal_compliant: false,
            landlord_knows_5_year_prior_flooding: false,
            dwelling_unit_count: 4,
            owner_occupied: false,
            commercial_property: false,
            disclosure_provided_before_lease: true,
            myhazards_url_included: true,
            insurance_recommendation_included: true,
            minimum_type_size_satisfied: true,
            nj_separate_rider_with_signature: true,
            tenant_substantial_loss_engaged: false,
            days_since_substantial_loss: 0,
        }
    }

    #[test]
    fn ca_special_flood_hazard_compliant() {
        let r = check(&ca_baseline());
        assert!(r.disclosure_obligation_triggered);
        assert!(r.disclosure_compliant);
        assert!(r.failure_reasons.is_empty());
    }

    #[test]
    fn ca_no_landlord_knowledge_no_obligation() {
        let mut i = ca_baseline();
        i.landlord_has_actual_knowledge = false;
        let r = check(&i);
        assert!(!r.disclosure_obligation_triggered);
    }

    #[test]
    fn ca_pre_july_2018_no_obligation() {
        let mut i = ca_baseline();
        i.lease_year = 2018;
        i.lease_month = 6;
        let r = check(&i);
        assert!(!r.disclosure_obligation_triggered);
        i.lease_month = 7;
        let r2 = check(&i);
        assert!(r2.disclosure_obligation_triggered);
    }

    #[test]
    fn ca_missing_disclosure_violation() {
        let mut i = ca_baseline();
        i.disclosure_provided_before_lease = false;
        let r = check(&i);
        assert!(r.disclosure_obligation_triggered);
        assert!(!r.disclosure_compliant);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 8589.45(a)")
            && f.contains("July 1, 2018")));
    }

    #[test]
    fn ca_missing_myhazards_url_violation() {
        let mut i = ca_baseline();
        i.myhazards_url_included = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 8589.45(a)(2)")
            && f.contains("MyHazards")));
    }

    #[test]
    fn ca_missing_insurance_recommendation_violation() {
        let mut i = ca_baseline();
        i.insurance_recommendation_included = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 8589.45(a)(3)")
            && f.contains("renter's insurance and flood insurance")));
    }

    #[test]
    fn ca_under_8_point_type_violation() {
        let mut i = ca_baseline();
        i.minimum_type_size_satisfied = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 8589.45(c)")
            && f.contains("8-point minimum")));
    }

    #[test]
    fn texas_floodplain_100_year_triggers_obligation() {
        let i = FloodHazardDisclosureInput {
            jurisdiction: Jurisdiction::Texas,
            lease_year: 2026,
            lease_month: 1,
            in_special_flood_hazard_area: true,
            in_moderate_risk_flood_area: false,
            landlord_has_actual_knowledge: false,
            elevation_above_floodplain_federal_compliant: false,
            landlord_knows_5_year_prior_flooding: false,
            dwelling_unit_count: 1,
            owner_occupied: false,
            commercial_property: false,
            disclosure_provided_before_lease: false,
            myhazards_url_included: false,
            insurance_recommendation_included: false,
            minimum_type_size_satisfied: false,
            nj_separate_rider_with_signature: false,
            tenant_substantial_loss_engaged: false,
            days_since_substantial_loss: 0,
        };
        let r = check(&i);
        assert!(r.disclosure_obligation_triggered);
        assert!(!r.disclosure_compliant);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 92.0135")));
    }

    #[test]
    fn texas_elevation_above_floodplain_exception_lifts_obligation() {
        let mut i = FloodHazardDisclosureInput {
            jurisdiction: Jurisdiction::Texas,
            lease_year: 2026,
            lease_month: 1,
            in_special_flood_hazard_area: true,
            in_moderate_risk_flood_area: false,
            landlord_has_actual_knowledge: false,
            elevation_above_floodplain_federal_compliant: true,
            landlord_knows_5_year_prior_flooding: false,
            dwelling_unit_count: 1,
            owner_occupied: false,
            commercial_property: false,
            disclosure_provided_before_lease: false,
            myhazards_url_included: false,
            insurance_recommendation_included: false,
            minimum_type_size_satisfied: false,
            nj_separate_rider_with_signature: false,
            tenant_substantial_loss_engaged: false,
            days_since_substantial_loss: 0,
        };
        let r = check(&i);
        assert!(!r.disclosure_obligation_triggered);
        i.in_special_flood_hazard_area = false;
        i.elevation_above_floodplain_federal_compliant = false;
        let r2 = check(&i);
        assert!(!r2.disclosure_obligation_triggered);
    }

    #[test]
    fn texas_5_year_prior_flooding_triggers_obligation() {
        let mut i = FloodHazardDisclosureInput {
            jurisdiction: Jurisdiction::Texas,
            lease_year: 2026,
            lease_month: 1,
            in_special_flood_hazard_area: false,
            in_moderate_risk_flood_area: false,
            landlord_has_actual_knowledge: false,
            elevation_above_floodplain_federal_compliant: false,
            landlord_knows_5_year_prior_flooding: true,
            dwelling_unit_count: 1,
            owner_occupied: false,
            commercial_property: false,
            disclosure_provided_before_lease: false,
            myhazards_url_included: false,
            insurance_recommendation_included: false,
            minimum_type_size_satisfied: false,
            nj_separate_rider_with_signature: false,
            tenant_substantial_loss_engaged: false,
            days_since_substantial_loss: 0,
        };
        let r = check(&i);
        assert!(r.disclosure_obligation_triggered);
        i.landlord_knows_5_year_prior_flooding = false;
        let r2 = check(&i);
        assert!(!r2.disclosure_obligation_triggered);
    }

    #[test]
    fn texas_substantial_loss_within_30_days_engages_termination_right() {
        let mut i = FloodHazardDisclosureInput {
            jurisdiction: Jurisdiction::Texas,
            lease_year: 2026,
            lease_month: 1,
            in_special_flood_hazard_area: true,
            in_moderate_risk_flood_area: false,
            landlord_has_actual_knowledge: true,
            elevation_above_floodplain_federal_compliant: false,
            landlord_knows_5_year_prior_flooding: true,
            dwelling_unit_count: 1,
            owner_occupied: false,
            commercial_property: false,
            disclosure_provided_before_lease: false,
            myhazards_url_included: false,
            insurance_recommendation_included: false,
            minimum_type_size_satisfied: false,
            nj_separate_rider_with_signature: false,
            tenant_substantial_loss_engaged: true,
            days_since_substantial_loss: 15,
        };
        let r = check(&i);
        assert!(r.tenant_lease_termination_right_engaged);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 92.0135(c)")
            && f.contains("TERMINATE LEASE within 30 days")));
        i.days_since_substantial_loss = 31;
        let r2 = check(&i);
        assert!(!r2.tenant_lease_termination_right_engaged);
    }

    #[test]
    fn texas_pre_2022_no_obligation() {
        let i = FloodHazardDisclosureInput {
            jurisdiction: Jurisdiction::Texas,
            lease_year: 2021,
            lease_month: 12,
            in_special_flood_hazard_area: true,
            in_moderate_risk_flood_area: false,
            landlord_has_actual_knowledge: false,
            elevation_above_floodplain_federal_compliant: false,
            landlord_knows_5_year_prior_flooding: true,
            dwelling_unit_count: 1,
            owner_occupied: false,
            commercial_property: false,
            disclosure_provided_before_lease: false,
            myhazards_url_included: false,
            insurance_recommendation_included: false,
            minimum_type_size_satisfied: false,
            nj_separate_rider_with_signature: false,
            tenant_substantial_loss_engaged: false,
            days_since_substantial_loss: 0,
        };
        let r = check(&i);
        assert!(!r.disclosure_obligation_triggered);
    }

    #[test]
    fn nj_3_unit_floodplain_triggers_obligation() {
        let mut i = ca_baseline();
        i.jurisdiction = Jurisdiction::NewJersey;
        i.dwelling_unit_count = 3;
        i.owner_occupied = false;
        i.lease_year = 2026;
        let r = check(&i);
        assert!(r.disclosure_obligation_triggered);
    }

    #[test]
    fn nj_2_unit_no_obligation() {
        let mut i = ca_baseline();
        i.jurisdiction = Jurisdiction::NewJersey;
        i.dwelling_unit_count = 2;
        i.owner_occupied = false;
        i.lease_year = 2026;
        let r = check(&i);
        assert!(!r.disclosure_obligation_triggered);
    }

    #[test]
    fn nj_owner_occupied_4_unit_triggers_obligation() {
        let mut i = ca_baseline();
        i.jurisdiction = Jurisdiction::NewJersey;
        i.dwelling_unit_count = 4;
        i.owner_occupied = true;
        i.lease_year = 2026;
        let r = check(&i);
        assert!(r.disclosure_obligation_triggered);
    }

    #[test]
    fn nj_owner_occupied_3_unit_no_obligation() {
        let mut i = ca_baseline();
        i.jurisdiction = Jurisdiction::NewJersey;
        i.dwelling_unit_count = 3;
        i.owner_occupied = true;
        i.lease_year = 2026;
        let r = check(&i);
        assert!(!r.disclosure_obligation_triggered);
    }

    #[test]
    fn nj_commercial_property_triggers_regardless_of_units() {
        let mut i = ca_baseline();
        i.jurisdiction = Jurisdiction::NewJersey;
        i.commercial_property = true;
        i.dwelling_unit_count = 1;
        i.lease_year = 2026;
        let r = check(&i);
        assert!(r.disclosure_obligation_triggered);
    }

    #[test]
    fn nj_pre_march_2024_no_obligation() {
        let mut i = ca_baseline();
        i.jurisdiction = Jurisdiction::NewJersey;
        i.dwelling_unit_count = 4;
        i.lease_year = 2024;
        i.lease_month = 2;
        let r = check(&i);
        assert!(!r.disclosure_obligation_triggered);
        i.lease_month = 3;
        let r2 = check(&i);
        assert!(r2.disclosure_obligation_triggered);
    }

    #[test]
    fn nj_moderate_risk_500_year_floodplain_triggers_obligation() {
        let mut i = ca_baseline();
        i.jurisdiction = Jurisdiction::NewJersey;
        i.dwelling_unit_count = 4;
        i.in_special_flood_hazard_area = false;
        i.in_moderate_risk_flood_area = true;
        let r = check(&i);
        assert!(r.disclosure_obligation_triggered);
    }

    #[test]
    fn nj_missing_rider_with_signature_violation() {
        let mut i = ca_baseline();
        i.jurisdiction = Jurisdiction::NewJersey;
        i.dwelling_unit_count = 4;
        i.nj_separate_rider_with_signature = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r.failure_reasons.iter().any(|f| f.contains("N.J.S.A. 46:8-50(c)")
            && f.contains("SEPARATE RIDER")));
    }

    #[test]
    fn nj_failure_to_disclose_engages_lease_termination_right() {
        let mut i = ca_baseline();
        i.jurisdiction = Jurisdiction::NewJersey;
        i.dwelling_unit_count = 4;
        i.disclosure_provided_before_lease = false;
        let r = check(&i);
        assert!(r.tenant_lease_termination_right_engaged);
        assert!(r.failure_reasons.iter().any(|f| f.contains("N.J.S.A. 46:8-50(d)")
            && f.contains("TERMINATE LEASE")));
    }

    #[test]
    fn default_no_obligation() {
        let mut i = ca_baseline();
        i.jurisdiction = Jurisdiction::Default;
        i.disclosure_provided_before_lease = false;
        let r = check(&i);
        assert!(!r.disclosure_obligation_triggered);
        assert!(r.disclosure_compliant);
    }

    #[test]
    fn citation_pins_all_four_jurisdictions() {
        let r = check(&ca_baseline());
        assert!(r.citation.contains("Cal. Gov. Code § 8589.45"));
        assert!(r.citation.contains("AB 646 of 2017"));
        assert!(r.citation.contains("Tex. Prop. Code § 92.0135"));
        assert!(r.citation.contains("HB 531 of 2021"));
        assert!(r.citation.contains("N.J.S.A. 46:8-50"));
        assert!(r.citation.contains("NJ Flood Risk Notification Law of 2024"));
        assert!(r.citation.contains("FEMA NFIP regulations 44 CFR § 59.1 and § 60.3"));
    }

    #[test]
    fn note_pins_ca_july_2018_effective_date() {
        let r = check(&ca_baseline());
        assert!(r.notes.iter().any(|n| n.contains("§ 8589.45")
            && n.contains("AB 646 of 2017")
            && n.contains("July 1, 2018")));
    }

    #[test]
    fn note_pins_ca_myhazards_website() {
        let r = check(&ca_baseline());
        assert!(r.notes.iter().any(|n| n.contains("§ 8589.45(a)(2)")
            && n.contains("MyHazards")));
    }

    #[test]
    fn note_pins_ca_renters_flood_insurance_recommendation() {
        let r = check(&ca_baseline());
        assert!(r.notes.iter().any(|n| n.contains("§ 8589.45(a)(3)")
            && n.contains("renter's AND flood insurance")));
    }

    #[test]
    fn note_pins_ca_8_point_type_requirement() {
        let r = check(&ca_baseline());
        assert!(r.notes.iter().any(|n| n.contains("§ 8589.45(c)")
            && n.contains("8-point MINIMUM")));
    }

    #[test]
    fn note_pins_texas_january_2022_two_prong_framework() {
        let r = check(&ca_baseline());
        assert!(r.notes.iter().any(|n| n.contains("§ 92.0135")
            && n.contains("HB 531 of 2021")
            && n.contains("January 1, 2022")
            && n.contains("100-year floodplain")
            && n.contains("5-year prior flooding")));
    }

    #[test]
    fn note_pins_texas_30_day_substantial_loss_remedy() {
        let r = check(&ca_baseline());
        assert!(r.notes.iter().any(|n| n.contains("§ 92.0135(c)")
            && n.contains("50%+ of personal property")
            && n.contains("30 DAYS OF LOSS")));
    }

    #[test]
    fn note_pins_nj_march_2024_effective_date_unit_threshold() {
        let r = check(&ca_baseline());
        assert!(r.notes.iter().any(|n| n.contains("N.J.S.A. 46:8-50")
            && n.contains("March 20, 2024")
            && n.contains("MORE THAN 2 units")
            && n.contains("500-year")));
    }

    #[test]
    fn note_pins_nj_12_point_rider_signature_requirement() {
        let r = check(&ca_baseline());
        assert!(r.notes.iter().any(|n| n.contains("N.J.S.A. 46:8-50(c)")
            && n.contains("SEPARATE RIDER")
            && n.contains("12-point MINIMUM")
            && n.contains("SIGNED OR ACKNOWLEDGED")));
    }

    #[test]
    fn note_pins_nj_lease_termination_remedy() {
        let r = check(&ca_baseline());
        assert!(r.notes.iter().any(|n| n.contains("N.J.S.A. 46:8-50(d)")
            && n.contains("TERMINATE LEASE")));
    }

    #[test]
    fn note_pins_fema_nfip_separate_from_state_disclosure() {
        let r = check(&ca_baseline());
        assert!(r.notes.iter().any(|n| n.contains("FEMA NFIP")
            && n.contains("44 CFR § 59.1")
            && n.contains("Special Flood Hazard Area")));
    }

    #[test]
    fn jurisdiction_truth_table_four_cells() {
        for jur in [
            Jurisdiction::California,
            Jurisdiction::Texas,
            Jurisdiction::NewJersey,
            Jurisdiction::Default,
        ] {
            let mut i = ca_baseline();
            i.jurisdiction = jur;
            i.dwelling_unit_count = 4;
            let r = check(&i);
            assert_eq!(r.jurisdiction, jur);
        }
    }

    #[test]
    fn nj_uniquely_engages_500_year_moderate_risk_invariant() {
        let mut ca = ca_baseline();
        ca.in_special_flood_hazard_area = false;
        ca.in_moderate_risk_flood_area = true;
        let r_ca = check(&ca);
        assert!(!r_ca.disclosure_obligation_triggered);

        let mut nj = ca_baseline();
        nj.jurisdiction = Jurisdiction::NewJersey;
        nj.dwelling_unit_count = 4;
        nj.in_special_flood_hazard_area = false;
        nj.in_moderate_risk_flood_area = true;
        let r_nj = check(&nj);
        assert!(r_nj.disclosure_obligation_triggered);
    }
}
