//! Tenant kitchen appliance replacement framework — covers landlord
//! continuing duty to provide and maintain refrigerator and stove under
//! California AB 628 (effective January 1, 2026) plus parallel state
//! habitability frameworks. Distinct from sibling [[tenant_in_unit_
//! appliance_repair_responsibility]] (broader landlord-provided appliance
//! repair duty), [[rental_in_unit_laundry_appliance_provision]] (iter 501
//! W/D framework), [[rental_gas_appliance_ban]] (electrification mandate
//! context for gas-stove replacement).
//!
//! Trader-landlord critical because (1) California AB 628 (Cal. Civ. Code
//! § 1941.1 amendment effective January 1, 2026) NEW MANDATE: refrigerator
//! plus stove must be provided and maintained in good working order for
//! leases entered, amended, or renewed on or after that date — first-of-its-
//! kind state-mandated appliance provision; (2) CPSC recall response window
//! is 30 days under AB 628 — failure to repair or replace recalled appliance
//! within 30 days breaches habitability; (3) refrigerator opt-out is
//! permitted ONLY by voluntary written tenant election at lease signing,
//! NOT mid-tenancy; (4) stove opt-out is NOT permitted regardless of
//! tenant preference; (5) exemptions: permanent supportive housing,
//! single-room occupancy (SRO), residential hotels, communal-kitchen
//! assisted living facilities; (6) energy-efficiency replacement triggers
//! state rebates (ENERGY STAR refrigerator + induction range incentives via
//! CalEnergy + Inflation Reduction Act § 50121 HOMES rebate + § 50122
//! HEEHRA rebate); (7) range hood ventilation per IRC M1503 prevents indoor
//! air quality degradation from gas-stove NO2 emissions.
//!
//! State frameworks:
//!
//! - **California** — Cal. Civ. Code § 1941.1 (amended by AB 628 effective
//!   January 1, 2026 — refrigerator + stove mandatory); Cal. Civ. Code
//!   § 1942 self-help repair after 30-day notice; Cal. Health & Safety Code
//!   § 17920.3 substandard housing classification; Cal. Civ. Code § 1946.7
//!   lease termination right.
//!
//! - **Massachusetts** — 105 CMR 410.100(E) Sanitary Code requires kitchen
//!   sink + range (cookstove); M.G.L. ch. 186 § 14 quiet enjoyment; M.G.L.
//!   ch. 111 § 127A enforcement; MA has REQUIRED stove provision since
//!   1976.
//!
//! - **New York** — NYC Admin Code § 27-2017.2 cooking-gas plus electrical
//!   requirements; MDL § 76 plumbing fixtures required; HPD inspection
//!   cycle.
//!
//! - **Illinois** — Chicago Municipal Code § 5-12-110 habitability
//!   requires kitchen with sink + stove + heat + cold water.
//!
//! - **Default** — common-law habitability doctrine plus state-specific
//!   lease requirements; appliance provision NOT statutorily mandated
//!   absent state habitability inclusion.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    California,
    Massachusetts,
    NewYork,
    Illinois,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplianceType {
    Refrigerator,
    Stove,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplianceCondition {
    /// Working condition.
    WorkingProperly,
    /// Failing or in disrepair past tenant-notice window.
    FailingPastNoticeWindow,
    /// CPSC-recalled appliance.
    CpscRecalled,
    /// Missing — not provided by landlord at all.
    NotProvided,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HousingScope {
    /// Standard residential rental — AB 628 + state habitability apply.
    StandardResidentialRental,
    /// Permanent supportive housing — AB 628 exempt.
    PermanentSupportiveHousingExempt,
    /// Single-room occupancy (SRO) — AB 628 exempt.
    SroSharedKitchenExempt,
    /// Residential hotel or assisted living with communal kitchen.
    ResidentialHotelCommunalKitchenExempt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    CompliantWorkingProperly,
    ExemptHousingType,
    RefrigeratorOptOutVoluntaryDocumented,
    StoveOptOutAttemptedProhibited,
    AppliancePastDueRepairOrReplacementHabitabilityBreach,
    CpscRecalledNotReplacedWithin30DaysHabitabilityBreach,
    NoApplianceProvidedAb628Violation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub housing_scope: HousingScope,
    pub appliance_type: ApplianceType,
    pub appliance_condition: ApplianceCondition,
    pub lease_year: i32,
    pub lease_entered_amended_or_renewed_on_or_after_2026: bool,
    pub tenant_signed_voluntary_refrigerator_opt_out: bool,
    pub days_since_cpsc_recall_notification: u32,
    pub annual_rent_cents: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub recommended_actions: Vec<String>,
    pub annual_rent_at_risk_cents: u64,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub const AB_628_EFFECTIVE_YEAR: i32 = 2026;
pub const AB_628_CPSC_RECALL_RESPONSE_DAYS: u32 = 30;
pub const MA_STOVE_PROVISION_SINCE_YEAR: i32 = 1976;

pub fn check(input: &Input) -> Output {
    let mut actions: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    let severity: Severity;

    if !matches!(input.housing_scope, HousingScope::StandardResidentialRental) {
        notes.push(
            "AB 628 exempts permanent supportive housing, single-room occupancy units, \
             residential hotels, and assisted-living facilities with communal kitchens \
             from the refrigerator + stove provision mandate. Exempt scope still subject \
             to state habitability framework for shared cooking facilities."
                .to_string(),
        );
        return Output {
            severity: Severity::ExemptHousingType,
            recommended_actions: actions,
            annual_rent_at_risk_cents: 0,
            citation: "Cal. Civ. Code § 1941.1(b) AB 628 exemption",
            notes,
        };
    }

    let ab_628_in_force = matches!(input.jurisdiction, Jurisdiction::California)
        && input.lease_entered_amended_or_renewed_on_or_after_2026
        && input.lease_year >= AB_628_EFFECTIVE_YEAR;

    if matches!(input.appliance_condition, ApplianceCondition::CpscRecalled)
        && input.days_since_cpsc_recall_notification > AB_628_CPSC_RECALL_RESPONSE_DAYS
    {
        severity = Severity::CpscRecalledNotReplacedWithin30DaysHabitabilityBreach;
        actions.push(format!(
            "CPSC-recalled appliance ({:?}) not repaired or replaced within {}-day window \
             per AB 628 (CA) — habitability breach plus consumer-protection violation. \
             Document CPSC recall notice, schedule immediate replacement with non-recalled \
             model, notify tenant in writing of replacement timeline, and verify recalled \
             appliance is rendered unusable to prevent tenant use during replacement \
             window.",
            input.appliance_type, AB_628_CPSC_RECALL_RESPONSE_DAYS
        ));
    } else if matches!(input.appliance_condition, ApplianceCondition::NotProvided) {
        if ab_628_in_force && matches!(input.appliance_type, ApplianceType::Stove) {
            severity = Severity::NoApplianceProvidedAb628Violation;
            actions.push(format!(
                "Stove NOT provided for lease entered/amended/renewed on or after January 1, \
                 {} — AB 628 violation (Cal. Civ. Code § 1941.1 amended); tenant CANNOT opt \
                 out of stove requirement. Procure and install working stove within 30 days; \
                 tenant entitled to rent withholding plus self-help installation plus damages \
                 until cured.",
                AB_628_EFFECTIVE_YEAR
            ));
        } else if ab_628_in_force
            && matches!(input.appliance_type, ApplianceType::Refrigerator)
            && !input.tenant_signed_voluntary_refrigerator_opt_out
        {
            severity = Severity::NoApplianceProvidedAb628Violation;
            actions.push(format!(
                "Refrigerator NOT provided for lease entered/amended/renewed on or after \
                 January 1, {} — AB 628 violation absent voluntary written tenant opt-out at \
                 lease signing. Procure and install working refrigerator within 30 days OR \
                 obtain documented tenant voluntary opt-out (must be at lease signing, not \
                 mid-tenancy).",
                AB_628_EFFECTIVE_YEAR
            ));
        } else if ab_628_in_force
            && matches!(input.appliance_type, ApplianceType::Refrigerator)
            && input.tenant_signed_voluntary_refrigerator_opt_out
        {
            severity = Severity::RefrigeratorOptOutVoluntaryDocumented;
            actions.push(
                "Refrigerator opt-out documented in writing by tenant at lease signing per \
                 AB 628 voluntary exception; tenant supplies own refrigerator. Retain signed \
                 opt-out lease addendum plus tenant acknowledgment in landlord file. Opt-out \
                 does NOT carry forward to subsequent tenants — re-acquire opt-out at each \
                 new lease."
                    .to_string(),
            );
        } else {
            severity = Severity::AppliancePastDueRepairOrReplacementHabitabilityBreach;
            actions.push(format!(
                "{:?} not provided. State habitability frameworks (105 CMR 410.100(E) MA + \
                 NYC Admin Code § 27-2017.2 NY + Chicago Municipal Code § 5-12-110 IL) \
                 require kitchen with stove. Procure and install working appliance.",
                input.appliance_type
            ));
        }
    } else if matches!(
        input.appliance_type,
        ApplianceType::Stove
    ) && input.tenant_signed_voluntary_refrigerator_opt_out
        && ab_628_in_force
    {
        severity = Severity::StoveOptOutAttemptedProhibited;
        actions.push(
            "Tenant attempted to opt out of STOVE provision — AB 628 PROHIBITS stove opt-out \
             regardless of tenant preference. Stove must be provided + maintained by \
             landlord; voluntary opt-out is available only for refrigerator. Reject any lease \
             clause attempting stove opt-out as void against public policy."
                .to_string(),
        );
    } else if matches!(
        input.appliance_condition,
        ApplianceCondition::FailingPastNoticeWindow
    ) {
        severity = Severity::AppliancePastDueRepairOrReplacementHabitabilityBreach;
        actions.push(
            "Landlord-provided appliance in disrepair past tenant-notice repair window — \
             habitability breach under Cal. Civ. Code § 1941 / § 1942 (CA), M.G.L. ch. 186 § \
             14 (MA), Real Property Law § 235-b (NY), or Chicago Municipal Code § 5-12-110 \
             (IL). Repair or replace within statutory time window (typically 30 days from \
             written notice); failure exposes landlord to tenant rent withholding + repair-\
             and-deduct + constructive-eviction claim."
                .to_string(),
        );
    } else {
        severity = Severity::CompliantWorkingProperly;
        actions.push(
            "Compliant: appliance working properly. Document with annual landlord inspection \
             plus tenant acknowledgment of working condition. Maintain manufacturer warranty \
             records plus appliance-replacement reserve fund of approximately $1,500 per \
             unit for end-of-life refrigerator replacement and $1,000 for stove replacement."
                .to_string(),
        );
    }

    match input.jurisdiction {
        Jurisdiction::California => {
            notes.push(format!(
                "Cal. Civ. Code § 1941.1 amended by AB 628 effective January 1, {} — \
                 refrigerator + stove are NEW MANDATORY habitability provisions; CPSC \
                 recall must be cured within {}-day window. Cal. Civ. Code § 1942 self-\
                 help repair after 30-day notice; Cal. Health & Safety Code § 17920.3 \
                 substandard housing classification. Pairs with Inflation Reduction Act \
                 § 50121 HOMES rebate + § 50122 HEEHRA rebate for ENERGY STAR + induction \
                 range upgrades.",
                AB_628_EFFECTIVE_YEAR, AB_628_CPSC_RECALL_RESPONSE_DAYS
            ));
        }
        Jurisdiction::Massachusetts => {
            notes.push(format!(
                "105 CMR 410.100(E) State Sanitary Code requires kitchen sink + range \
                 (cookstove); M.G.L. ch. 186 § 14 quiet enjoyment; M.G.L. ch. 111 § 127A \
                 enforcement. Massachusetts has REQUIRED stove provision since {}; \
                 refrigerator provision is market-driven, not statutorily mandated.",
                MA_STOVE_PROVISION_SINCE_YEAR
            ));
        }
        Jurisdiction::NewYork => {
            notes.push(
                "NYC Admin Code § 27-2017.2 requires cooking-gas plus electrical service \
                 capability for kitchen; MDL § 76 plumbing fixtures required; HPD \
                 inspection cycle enforces. NYS Real Property Law § 235-b implied \
                 warranty of habitability extends to provided appliances."
                    .to_string(),
            );
        }
        Jurisdiction::Illinois => {
            notes.push(
                "Chicago Municipal Code § 5-12-110 Residential Landlord and Tenant \
                 Ordinance (RLTO) habitability standard requires kitchen with sink + \
                 stove + heat + cold water. Outside Chicago, Illinois Mobile Home \
                 Landlord and Tenant Rights Act and common-law habitability govern."
                    .to_string(),
            );
        }
        Jurisdiction::Default => {
            notes.push(
                "Common-law implied warranty of habitability plus state-specific lease \
                 requirements; appliance provision NOT statutorily mandated absent state \
                 habitability inclusion. Federal CPSC recall framework (15 U.S.C. § 2064 \
                 substantial product hazard) applies in all jurisdictions for recalled \
                 appliances."
                    .to_string(),
            );
        }
    }

    notes.push(
        "Coordination with [[tenant_in_unit_appliance_repair_responsibility]] (broader \
         landlord-provided appliance maintenance duty), [[rental_in_unit_laundry_appliance_\
         provision]] (iter 501 W/D framework), [[rental_gas_appliance_ban]] (electrification \
         mandate context — converting gas stove to induction range qualifies for IRA HEEHRA \
         rebate), [[rental_natural_gas_leak_response]] (gas stove leak protocol), \
         [[mid_tenancy_temporary_relocation]] (when tenant must vacate during major \
         appliance replacement), [[tenant_emotional_distress_damages]] (IIED claim for \
         willful refusal to replace failing appliance during extended habitability breach)."
            .to_string(),
    );

    let annual_rent_at_risk: u64 = match severity {
        Severity::CpscRecalledNotReplacedWithin30DaysHabitabilityBreach
        | Severity::NoApplianceProvidedAb628Violation
        | Severity::AppliancePastDueRepairOrReplacementHabitabilityBreach => {
            input.annual_rent_cents
        }
        Severity::StoveOptOutAttemptedProhibited => {
            input.annual_rent_cents.saturating_div(2)
        }
        _ => 0,
    };

    Output {
        severity,
        recommended_actions: actions,
        annual_rent_at_risk_cents: annual_rent_at_risk,
        citation: match input.jurisdiction {
            Jurisdiction::California => "Cal. Civ. Code § 1941.1 + § 1942 + AB 628 + § 17920.3",
            Jurisdiction::Massachusetts => "105 CMR 410.100(E) + M.G.L. ch. 186 § 14 + ch. 111 § 127A",
            Jurisdiction::NewYork => "NYC Admin § 27-2017.2 + MDL § 76 + RPL § 235-b",
            Jurisdiction::Illinois => "Chicago Municipal Code § 5-12-110 RLTO + common-law habitability",
            Jurisdiction::Default => "Common-law habitability + 15 U.S.C. § 2064 CPSC",
        },
        notes,
    }
}

pub type TenantKitchenApplianceReplacementInput = Input;
pub type TenantKitchenApplianceReplacementResult = Output;

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Input {
        Input {
            jurisdiction: Jurisdiction::California,
            housing_scope: HousingScope::StandardResidentialRental,
            appliance_type: ApplianceType::Refrigerator,
            appliance_condition: ApplianceCondition::WorkingProperly,
            lease_year: 2026,
            lease_entered_amended_or_renewed_on_or_after_2026: true,
            tenant_signed_voluntary_refrigerator_opt_out: false,
            days_since_cpsc_recall_notification: 0,
            annual_rent_cents: 36_000_00,
        }
    }

    #[test]
    fn standard_rental_working_refrigerator_compliant() {
        let i = baseline();
        let r = check(&i);
        assert!(matches!(r.severity, Severity::CompliantWorkingProperly));
        assert_eq!(r.annual_rent_at_risk_cents, 0);
    }

    #[test]
    fn permanent_supportive_housing_exempt() {
        let mut i = baseline();
        i.housing_scope = HousingScope::PermanentSupportiveHousingExempt;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::ExemptHousingType));
        assert!(r.notes.iter().any(|n| n.contains("AB 628 exempt")));
    }

    #[test]
    fn sro_shared_kitchen_exempt() {
        let mut i = baseline();
        i.housing_scope = HousingScope::SroSharedKitchenExempt;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::ExemptHousingType));
    }

    #[test]
    fn residential_hotel_communal_kitchen_exempt() {
        let mut i = baseline();
        i.housing_scope = HousingScope::ResidentialHotelCommunalKitchenExempt;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::ExemptHousingType));
    }

    #[test]
    fn ca_no_stove_post_2026_ab_628_violation() {
        let mut i = baseline();
        i.appliance_type = ApplianceType::Stove;
        i.appliance_condition = ApplianceCondition::NotProvided;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NoApplianceProvidedAb628Violation));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents);
        assert!(r.recommended_actions.iter().any(|a| a.contains("CANNOT opt")));
    }

    #[test]
    fn ca_no_refrigerator_no_opt_out_violation() {
        let mut i = baseline();
        i.appliance_type = ApplianceType::Refrigerator;
        i.appliance_condition = ApplianceCondition::NotProvided;
        i.tenant_signed_voluntary_refrigerator_opt_out = false;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NoApplianceProvidedAb628Violation));
    }

    #[test]
    fn ca_no_refrigerator_with_voluntary_opt_out_documented() {
        let mut i = baseline();
        i.appliance_type = ApplianceType::Refrigerator;
        i.appliance_condition = ApplianceCondition::NotProvided;
        i.tenant_signed_voluntary_refrigerator_opt_out = true;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::RefrigeratorOptOutVoluntaryDocumented
        ));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("opt-out lease addendum")));
        assert_eq!(r.annual_rent_at_risk_cents, 0);
    }

    #[test]
    fn ca_stove_opt_out_attempted_prohibited() {
        let mut i = baseline();
        i.appliance_type = ApplianceType::Stove;
        i.appliance_condition = ApplianceCondition::WorkingProperly;
        i.tenant_signed_voluntary_refrigerator_opt_out = true;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::StoveOptOutAttemptedProhibited));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents / 2);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("void against public policy")));
    }

    #[test]
    fn cpsc_recalled_under_30_days_compliant_path() {
        let mut i = baseline();
        i.appliance_condition = ApplianceCondition::CpscRecalled;
        i.days_since_cpsc_recall_notification = 15;
        let r = check(&i);
        assert!(!matches!(
            r.severity,
            Severity::CpscRecalledNotReplacedWithin30DaysHabitabilityBreach
        ));
    }

    #[test]
    fn cpsc_recalled_over_30_days_habitability_breach() {
        let mut i = baseline();
        i.appliance_condition = ApplianceCondition::CpscRecalled;
        i.days_since_cpsc_recall_notification = 45;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::CpscRecalledNotReplacedWithin30DaysHabitabilityBreach
        ));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents);
    }

    #[test]
    fn cpsc_recalled_at_exactly_30_days_compliant() {
        let mut i = baseline();
        i.appliance_condition = ApplianceCondition::CpscRecalled;
        i.days_since_cpsc_recall_notification = 30;
        let r = check(&i);
        assert!(!matches!(
            r.severity,
            Severity::CpscRecalledNotReplacedWithin30DaysHabitabilityBreach
        ));
    }

    #[test]
    fn failing_appliance_past_notice_habitability_breach() {
        let mut i = baseline();
        i.appliance_condition = ApplianceCondition::FailingPastNoticeWindow;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::AppliancePastDueRepairOrReplacementHabitabilityBreach
        ));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents);
    }

    #[test]
    fn pre_2026_ca_lease_no_ab_628_mandate() {
        let mut i = baseline();
        i.appliance_type = ApplianceType::Stove;
        i.appliance_condition = ApplianceCondition::NotProvided;
        i.lease_year = 2025;
        i.lease_entered_amended_or_renewed_on_or_after_2026 = false;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::AppliancePastDueRepairOrReplacementHabitabilityBreach
        ));
    }

    #[test]
    fn ab_628_effective_year_pins_2026() {
        assert_eq!(AB_628_EFFECTIVE_YEAR, 2026);
    }

    #[test]
    fn ab_628_cpsc_recall_response_days_pins_30() {
        assert_eq!(AB_628_CPSC_RECALL_RESPONSE_DAYS, 30);
    }

    #[test]
    fn ma_stove_provision_since_year_pins_1976() {
        assert_eq!(MA_STOVE_PROVISION_SINCE_YEAR, 1976);
    }

    #[test]
    fn ma_jurisdiction_pins_105_cmr_410_100_e_and_stove_since_1976() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Massachusetts;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("105 CMR 410.100(E)")));
        assert!(r.notes.iter().any(|n| n.contains("1976")));
        assert!(r.notes.iter().any(|n| n.contains("M.G.L. ch. 111 § 127A")));
    }

    #[test]
    fn ny_jurisdiction_pins_nyc_admin_27_2017_2_and_mdl_76() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::NewYork;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("§ 27-2017.2")));
        assert!(r.notes.iter().any(|n| n.contains("MDL § 76")));
        assert!(r.notes.iter().any(|n| n.contains("§ 235-b")));
    }

    #[test]
    fn il_jurisdiction_pins_chicago_5_12_110_rlto() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Illinois;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("Chicago Municipal Code § 5-12-110")));
        assert!(r.notes.iter().any(|n| n.contains("RLTO")));
    }

    #[test]
    fn ca_jurisdiction_pins_ab_628_and_heehra_rebate() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("AB 628")));
        assert!(r.notes.iter().any(|n| n.contains("HEEHRA")));
        assert!(r.notes.iter().any(|n| n.contains("HOMES")));
        assert!(r.notes.iter().any(|n| n.contains("§ 50121")));
        assert!(r.notes.iter().any(|n| n.contains("§ 50122")));
    }

    #[test]
    fn default_jurisdiction_pins_cpsc_15_usc_2064() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Default;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("15 U.S.C. § 2064")));
        assert!(r.notes.iter().any(|n| n.contains("CPSC")));
    }

    #[test]
    fn coordination_note_references_siblings() {
        let i = baseline();
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("tenant_in_unit_appliance_repair_responsibility")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("rental_in_unit_laundry_appliance_provision")));
        assert!(r.notes.iter().any(|n| n.contains("rental_gas_appliance_ban")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("rental_natural_gas_leak_response")));
    }

    #[test]
    fn coordination_note_pinned_in_all_jurisdictions() {
        for j in [
            Jurisdiction::California,
            Jurisdiction::Massachusetts,
            Jurisdiction::NewYork,
            Jurisdiction::Illinois,
            Jurisdiction::Default,
        ] {
            let mut i = baseline();
            i.jurisdiction = j;
            let r = check(&i);
            assert!(
                r.notes
                    .iter()
                    .any(|n| n.contains("tenant_in_unit_appliance_repair_responsibility")),
                "coordination missing for {j:?}"
            );
        }
    }

    #[test]
    fn citation_branch_for_each_jurisdiction() {
        let ca = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::California; i });
        let ma = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::Massachusetts; i });
        let ny = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::NewYork; i });
        let il = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::Illinois; i });
        let de = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::Default; i });
        assert!(ca.citation.contains("AB 628"));
        assert!(ma.citation.contains("105 CMR 410.100(E)"));
        assert!(ny.citation.contains("§ 27-2017.2"));
        assert!(il.citation.contains("§ 5-12-110"));
        assert!(de.citation.contains("15 U.S.C. § 2064"));
    }

    #[test]
    fn zero_rent_zero_risk_does_not_panic() {
        let mut i = baseline();
        i.annual_rent_cents = 0;
        i.appliance_type = ApplianceType::Stove;
        i.appliance_condition = ApplianceCondition::NotProvided;
        let r = check(&i);
        assert_eq!(r.annual_rent_at_risk_cents, 0);
    }

    #[test]
    fn severity_priority_exempt_overrides_failing_appliance() {
        let mut i = baseline();
        i.housing_scope = HousingScope::PermanentSupportiveHousingExempt;
        i.appliance_condition = ApplianceCondition::FailingPastNoticeWindow;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::ExemptHousingType));
    }

    #[test]
    fn severity_priority_cpsc_overrides_not_provided() {
        let mut i = baseline();
        i.appliance_condition = ApplianceCondition::CpscRecalled;
        i.days_since_cpsc_recall_notification = 60;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::CpscRecalledNotReplacedWithin30DaysHabitabilityBreach
        ));
    }
}
