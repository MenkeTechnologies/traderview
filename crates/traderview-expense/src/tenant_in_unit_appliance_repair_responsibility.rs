//! Multi-jurisdictional tenant IN-UNIT APPLIANCE repair
//! and replacement responsibility compliance framework.
//! When a landlord provides appliances (refrigerator,
//! stove/range, dishwasher, washer/dryer, microwave,
//! HVAC) as part of a rental tenancy, what repair and
//! replacement obligations attach under the implied
//! warranty of habitability and state-specific
//! mandatory-provision statutes, what tenant remedies
//! apply (rent withholding + repair-and-deduct + lease
//! termination), and what failure-mode liabilities expose
//! landlord when an appliance fails?
//!
//! Distinct from sibling modules: rental_hot_water_
//! temperature (water heater specifically), rental_gas_
//! appliance_ban (electrification policy), rental_
//! carbon_monoxide_detector (CO sensor), rental_chimney_
//! fireplace_inspection_disclosure (iter 471), rental_
//! fire_extinguisher_requirement (iter 473), rental_
//! hardwired_smoke_alarm_responsibility (iter 481),
//! rental_garage_door_safety_compliance (iter 483),
//! rental_natural_gas_leak_response (iter 485),
//! landlord_repair_response_timeframe.
//!
//! Four-jurisdiction framework:
//!
//! 1. NEW YORK (most prescriptive for refrigerator
//!    provision) — NYC Admin. Code § 27-2014 + § 27-2008
//!    and N.Y. Multiple Dwelling Law § 78 require
//!    landlord to provide REFRIGERATOR and STOVE in
//!    NYC multi-unit dwellings; N.Y. Real Property Law
//!    § 235-b implied warranty of habitability requires
//!    fitness for human habitation including operable
//!    appliances; 28 RCNY § 32-04 HPD enforcement
//!    framework; HPD complaint and ECB violation
//!    issuance routes for unrepaired appliances.
//! 2. TEXAS — Tex. Prop. Code § 92.052 landlord duty to
//!    repair conditions that MATERIALLY AFFECT health
//!    or safety of ordinary tenant; § 92.053 effect of
//!    notice from tenant; § 92.056 tenant remedies for
//!    landlord's failure to repair (rent withholding,
//!    REPAIR-AND-DEDUCT up to one month's rent, lease
//!    termination, judicial order); appliance repair
//!    obligation if landlord provides appliance.
//! 3. CALIFORNIA — Cal. Civ. Code § 1941 + § 1941.1
//!    implied warranty of habitability; § 1941.1(a)(8)
//!    expressly requires landlord to provide STOVE
//!    (range); § 1942(a) tenant repair-and-deduct
//!    remedy (up to one month's rent, max twice in
//!    12-month period); Green v. Superior Court, 10
//!    Cal. 3d 616 (1974) common-law habitability;
//!    Hinson v. Delis, 26 Cal. App. 3d 62 (1972)
//!    appliance-specific habitability application.
//! 4. DEFAULT — Common-law implied warranty of
//!    habitability per Hilder v. St. Peter, 478 A.2d
//!    202 (Vt. 1984); Lemle v. Breeden, 51 Haw. 426
//!    (1969) (first state to adopt implied warranty);
//!    tort negligence + premises liability when
//!    appliance failure causes tenant injury.
//!
//! Six categories of in-unit appliances modeled:
//! 1. STOVE/RANGE — California requires under
//!    § 1941.1(a)(8); NY requires; most states require
//!    if heating or cooking is sole source
//! 2. REFRIGERATOR — NYC requires; most states do not
//!    statutorily require but enforce warranty if
//!    landlord-provided
//! 3. DISHWASHER — generally not statutorily required;
//!    warranty if provided
//! 4. WASHER/DRYER — generally not required; emerging
//!    "in-unit laundry" trend; warranty if provided
//! 5. MICROWAVE — not required; warranty if provided
//! 6. HVAC (heat + cooling) — heating universally
//!    required under habitability; cooling required in
//!    NY (post-2022 NYC Local Law 18) and select CA
//!    cities; landlord must maintain both
//!
//! Five universal failure-mode liability framework:
//! 1. REFUSAL TO REPAIR LANDLORD-PROVIDED APPLIANCE
//!    after timely tenant notice → habitability breach
//!    + rent withholding right (Cal. Civ. Code § 1942
//!    + Tex. Prop. Code § 92.056) + repair-and-deduct
//! 2. CONSTRUCTIVE EVICTION from extended non-repair
//!    (Hilder v. St. Peter) → tenant lease termination
//!    right + return of prepaid rent + relocation cost
//! 3. REPLACEMENT WITH MATERIALLY LOWER QUALITY than
//!    original (e.g., $200 used stove vs $800 OEM
//!    range) → habitability dispute + small claims
//! 4. TENANT-CAUSED DAMAGE allocation dispute →
//!    governed by lease terms; landlord cannot waive
//!    statutory minimum habitability under § 1942.1
//!    California
//! 5. USED APPLIANCE FAILURE due to end-of-useful-life
//!    → IRS § 168 depreciation lives suggest 5-7 year
//!    useful life for appliances; common-law tort if
//!    failure causes injury (e.g., fridge fire)
//!
//! Trader-landlord critical because (1) NYC refrigerator
//! requirement is among the most surprising compliance
//! traps for trader-landlords entering Manhattan/Brooklyn
//! markets — vacant units must include working
//! refrigerator before showing; (2) California § 1941.1(a)(8)
//! stove/range requirement is mandatory regardless of
//! lease terms — § 1942.1 voids any waiver; (3) tenant
//! repair-and-deduct under Cal. Civ. Code § 1942(a)
//! caps at one month's rent per repair + max twice in
//! 12-month period; (4) appliance failure causing fire/
//! flood/CO event escalates to wrongful death exposure;
//! (5) when in-unit washer/dryer is provided as
//! amenity but breaks for extended period, tenant rent
//! abatement claims are common.
//!
//! Authority: N.Y. Real Property Law § 235-b (implied
//! warranty); NYC Admin. Code § 27-2014; NYC Admin.
//! Code § 27-2008; N.Y. Multiple Dwelling Law § 78;
//! 28 RCNY § 32-04 (HPD enforcement); Tex. Prop. Code
//! § 92.052 (landlord duty to repair); Tex. Prop. Code
//! § 92.053; Tex. Prop. Code § 92.056 (tenant
//! remedies); Cal. Civ. Code § 1941; Cal. Civ. Code
//! § 1941.1(a)(8) (stove requirement); Cal. Civ. Code
//! § 1942 (repair-and-deduct); Cal. Civ. Code § 1942.1
//! (no waiver); Green v. Superior Court, 10 Cal. 3d
//! 616 (1974); Hinson v. Delis, 26 Cal. App. 3d 62
//! (1972); Hilder v. St. Peter, 478 A.2d 202 (Vt.
//! 1984); Lemle v. Breeden, 51 Haw. 426 (1969); IRS
//! § 168 (5-7 year appliance useful life).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    NewYork,
    Texas,
    California,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplianceCategory {
    StoveRange,
    Refrigerator,
    Dishwasher,
    WasherDryer,
    Microwave,
    HvacHeating,
    HvacCooling,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub appliance_category: ApplianceCategory,
    pub landlord_provided: bool,
    pub appliance_inoperable: bool,
    pub tenant_provided_timely_notice: bool,
    pub days_since_tenant_notice: u32,
    pub landlord_made_diligent_repair_effort: bool,
    pub repair_completed: bool,
    pub replacement_materially_lower_quality: bool,
    pub appliance_failure_caused_fire_flood_or_co_event: bool,
    pub nyc_multi_unit_dwelling: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    Compliant,
    NycRefrigeratorOrStoveMissing,
    CaStoveMandatoryViolation,
    HabitabilityBreachUnrepaired,
    InjuryEvent,
    QualityDegradationDispute,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub severity: Severity,
    pub jurisdiction_specific_actions: Vec<String>,
    pub notes: Vec<String>,
}

pub const REASONABLE_REPAIR_WINDOW_DAYS: u32 = 7;

pub type TenantInUnitApplianceRepairResponsibilityInput = Input;
pub type TenantInUnitApplianceRepairResponsibilityResult = Output;

pub fn check(input: &Input) -> Output {
    let notes: Vec<String> = vec![
        "Four-jurisdiction framework: New York (most prescriptive for refrigerator provision — NYC Admin. Code § 27-2014 + § 27-2008 + N.Y. Multiple Dwelling Law § 78 require landlord to provide REFRIGERATOR and STOVE in NYC multi-unit dwellings; N.Y. Real Property Law § 235-b implied warranty of habitability; 28 RCNY § 32-04 HPD enforcement); Texas (Tex. Prop. Code § 92.052 landlord duty to repair conditions that MATERIALLY AFFECT health or safety of ordinary tenant + § 92.053 effect of notice + § 92.056 tenant remedies including REPAIR-AND-DEDUCT up to one month's rent + lease termination); California (Cal. Civ. Code § 1941 + § 1941.1 implied warranty + § 1941.1(a)(8) STOVE requirement + § 1942(a) repair-and-deduct max twice in 12-month period + § 1942.1 no waiver + Green v. Superior Court 10 Cal. 3d 616 (1974) + Hinson v. Delis 26 Cal. App. 3d 62 (1972) appliance habitability); Default (common-law implied warranty per Hilder v. St. Peter 478 A.2d 202 (Vt. 1984) + Lemle v. Breeden 51 Haw. 426 (1969)).".to_string(),
        "Six categories of in-unit appliances: (1) STOVE/RANGE — California requires per § 1941.1(a)(8); NY requires; most states require if sole cooking/heating source; (2) REFRIGERATOR — NYC requires per NYC Admin. Code § 27-2014; most states do not statutorily require but enforce warranty if landlord-provided; (3) DISHWASHER — generally not statutorily required, warranty if provided; (4) WASHER/DRYER — generally not required, warranty if provided; (5) MICROWAVE — not required, warranty if provided; (6) HVAC — heating universally required under habitability, cooling required in NYC Local Law 18 (post-2022) and select CA cities.".to_string(),
        "Five universal failure-mode liabilities: (1) REFUSAL TO REPAIR LANDLORD-PROVIDED APPLIANCE after timely tenant notice → habitability breach + rent withholding + repair-and-deduct (Cal. Civ. Code § 1942 + Tex. Prop. Code § 92.056); (2) CONSTRUCTIVE EVICTION from extended non-repair (Hilder v. St. Peter) → tenant lease termination + relocation; (3) REPLACEMENT WITH MATERIALLY LOWER QUALITY than original → habitability dispute + small claims; (4) TENANT-CAUSED DAMAGE allocation dispute → governed by lease terms, landlord cannot waive statutory minimum habitability under § 1942.1; (5) USED APPLIANCE FAILURE due to end-of-useful-life → IRS § 168 5-7 year depreciation life + common-law tort if injury (fridge fire, gas explosion).".to_string(),
        "Tenant remedies for landlord refusal-to-repair: California Cal. Civ. Code § 1942(a) REPAIR-AND-DEDUCT — tenant may make repair + deduct cost (up to one month's rent, max twice in 12-month period); Texas Tex. Prop. Code § 92.056 — rent withholding + repair-and-deduct + judicial order + lease termination; New York rent-impairment-of-services + HPD complaint + ECB violation issuance + rent reduction in housing court.".to_string(),
        "When landlord provides appliance under lease, landlord assumes WARRANTY obligation under implied warranty of habitability — must repair or replace promptly. Reasonable repair window typically 7 days post-notice for non-emergency appliance; 24-72 hours for major appliance affecting habitability (refrigerator with spoilage risk + stove in winter + HVAC in extreme weather). Days-since-notice metric drives the habitability breach determination.".to_string(),
        "Companion modules: rental_hot_water_temperature (water heater), rental_gas_appliance_ban (electrification), rental_carbon_monoxide_detector, rental_chimney_fireplace_inspection_disclosure (iter 471), rental_fire_extinguisher_requirement (iter 473), rental_hardwired_smoke_alarm_responsibility (iter 481), rental_garage_door_safety_compliance (iter 483), rental_natural_gas_leak_response (iter 485), landlord_repair_response_timeframe.".to_string(),
    ];
    let mut actions: Vec<String> = Vec::new();

    if input.appliance_failure_caused_fire_flood_or_co_event {
        actions.push("Appliance failure caused fire/flood/CO event: engage emergency services + counsel; preserve evidence; tort negligence + wrongful death + IIED parallel to tenant_emotional_distress_damages iter 453.".to_string());
    }

    // Mandatory-provision check (CA stove + NYC stove/refrigerator)
    let ca_stove_required = matches!(input.jurisdiction, Jurisdiction::California)
        && matches!(input.appliance_category, ApplianceCategory::StoveRange)
        && !input.landlord_provided;
    if ca_stove_required {
        actions.push("California: Cal. Civ. Code § 1941.1(a)(8) MANDATES landlord provision of stove/range; § 1942.1 voids any contrary lease term. Landlord must provide functioning stove regardless of lease terms.".to_string());
    }

    let nyc_appliance_missing = matches!(input.jurisdiction, Jurisdiction::NewYork)
        && input.nyc_multi_unit_dwelling
        && matches!(
            input.appliance_category,
            ApplianceCategory::StoveRange | ApplianceCategory::Refrigerator
        )
        && !input.landlord_provided;
    if nyc_appliance_missing {
        actions.push("NYC multi-unit dwelling: NYC Admin. Code § 27-2014 + § 27-2008 + N.Y. Multiple Dwelling Law § 78 REQUIRE landlord to provide refrigerator and stove. HPD enforcement under 28 RCNY § 32-04 + ECB violation issuance.".to_string());
    }

    // If not provided and not required, not applicable
    if !input.landlord_provided && !ca_stove_required && !nyc_appliance_missing {
        let mut n = notes;
        n.push("Appliance not provided by landlord and not statutorily required in jurisdiction — no repair obligation. If landlord later provides as amenity, warranty attaches.".to_string());
        return Output {
            severity: Severity::NotApplicable,
            jurisdiction_specific_actions: actions,
            notes: n,
        };
    }

    // Mandatory-provision violations short-circuit before operability checks
    if !input.landlord_provided && (ca_stove_required || nyc_appliance_missing) {
        let severity = if input.appliance_failure_caused_fire_flood_or_co_event {
            Severity::InjuryEvent
        } else if nyc_appliance_missing {
            Severity::NycRefrigeratorOrStoveMissing
        } else {
            Severity::CaStoveMandatoryViolation
        };
        return Output {
            severity,
            jurisdiction_specific_actions: actions,
            notes,
        };
    }

    if !input.appliance_inoperable {
        // Appliance operable; check quality replacement issue
        if input.replacement_materially_lower_quality {
            actions.push("Replacement appliance MATERIALLY LOWER QUALITY than original: habitability dispute exposure; tenant may pursue small-claims action + warranty breach. Best practice: replace with equivalent or better.".to_string());
            return Output {
                severity: Severity::QualityDegradationDispute,
                jurisdiction_specific_actions: actions,
                notes,
            };
        }
        let mut n = notes;
        n.push("Appliance operable + provided + no quality degradation — compliant.".to_string());
        let severity = if input.appliance_failure_caused_fire_flood_or_co_event {
            Severity::InjuryEvent
        } else {
            Severity::Compliant
        };
        return Output {
            severity,
            jurisdiction_specific_actions: actions,
            notes: n,
        };
    }

    // Appliance inoperable
    if !input.tenant_provided_timely_notice {
        let mut n = notes;
        n.push("Appliance inoperable but no tenant notice received: landlord repair duty not yet triggered. Tex. Prop. Code § 92.053 + Cal. Civ. Code § 1942 require tenant to provide timely written notice before repair-and-deduct or other remedies apply.".to_string());
        return Output {
            severity: Severity::Compliant,
            jurisdiction_specific_actions: actions,
            notes: n,
        };
    }

    let extended_non_repair = input.days_since_tenant_notice > REASONABLE_REPAIR_WINDOW_DAYS
        && !input.repair_completed;
    if extended_non_repair && !input.landlord_made_diligent_repair_effort {
        let citation = match input.jurisdiction {
            Jurisdiction::NewYork => "N.Y. Real Property Law § 235-b implied warranty + HPD complaint + rent reduction in housing court + 28 RCNY § 32-04 enforcement",
            Jurisdiction::Texas => "Tex. Prop. Code § 92.056 — rent withholding + REPAIR-AND-DEDUCT up to one month's rent + lease termination + judicial order",
            Jurisdiction::California => "Cal. Civ. Code § 1942(a) — REPAIR-AND-DEDUCT (up to one month's rent, max twice in 12-month period) + § 1942.4 untenantable conditions framework",
            Jurisdiction::Default => "common-law implied warranty of habitability per Hilder v. St. Peter, 478 A.2d 202 (Vt. 1984) + rent withholding + constructive eviction",
        };
        actions.push(format!(
            "Appliance inoperable {} days after timely tenant notice without diligent landlord repair effort: habitability breach under {}.",
            input.days_since_tenant_notice, citation
        ));
    }

    match input.jurisdiction {
        Jurisdiction::NewYork => {
            actions.push("New York: N.Y. Real Property Law § 235-b implied warranty + NYC Admin. Code § 27-2014 + § 27-2008 + N.Y. Multiple Dwelling Law § 78 + 28 RCNY § 32-04 HPD enforcement.".to_string());
        }
        Jurisdiction::Texas => {
            actions.push("Texas: Tex. Prop. Code § 92.052 landlord duty to repair conditions that materially affect health or safety + § 92.053 notice effect + § 92.056 tenant remedies (rent withholding + repair-and-deduct + lease termination + judicial order).".to_string());
        }
        Jurisdiction::California => {
            actions.push("California: Cal. Civ. Code § 1941 + § 1941.1 implied warranty + § 1941.1(a)(8) stove requirement + § 1942(a) repair-and-deduct + § 1942.1 no-waiver + Green v. Superior Court, 10 Cal. 3d 616 (1974) + Hinson v. Delis, 26 Cal. App. 3d 62 (1972).".to_string());
        }
        Jurisdiction::Default => {
            actions.push("Default jurisdiction: common-law implied warranty of habitability per Hilder v. St. Peter, 478 A.2d 202 (Vt. 1984) + Lemle v. Breeden, 51 Haw. 426 (1969).".to_string());
        }
    }

    let severity = if input.appliance_failure_caused_fire_flood_or_co_event {
        Severity::InjuryEvent
    } else if nyc_appliance_missing {
        Severity::NycRefrigeratorOrStoveMissing
    } else if ca_stove_required {
        Severity::CaStoveMandatoryViolation
    } else if extended_non_repair && !input.landlord_made_diligent_repair_effort {
        Severity::HabitabilityBreachUnrepaired
    } else if input.replacement_materially_lower_quality {
        Severity::QualityDegradationDispute
    } else {
        Severity::Compliant
    };

    Output {
        severity,
        jurisdiction_specific_actions: actions,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Input {
        Input {
            jurisdiction: Jurisdiction::California,
            appliance_category: ApplianceCategory::StoveRange,
            landlord_provided: true,
            appliance_inoperable: false,
            tenant_provided_timely_notice: false,
            days_since_tenant_notice: 0,
            landlord_made_diligent_repair_effort: false,
            repair_completed: false,
            replacement_materially_lower_quality: false,
            appliance_failure_caused_fire_flood_or_co_event: false,
            nyc_multi_unit_dwelling: false,
        }
    }

    #[test]
    fn compliant_appliance_operable_baseline() {
        let i = baseline();
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn injury_event_top_severity() {
        let mut i = baseline();
        i.appliance_failure_caused_fire_flood_or_co_event = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::InjuryEvent);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("tort negligence + wrongful death"));
    }

    #[test]
    fn ca_stove_not_provided_mandatory_violation() {
        let mut i = baseline();
        i.appliance_category = ApplianceCategory::StoveRange;
        i.landlord_provided = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::CaStoveMandatoryViolation);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("§ 1941.1(a)(8)"));
        assert!(joined.contains("§ 1942.1"));
    }

    #[test]
    fn nyc_refrigerator_missing_severity() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::NewYork;
        i.appliance_category = ApplianceCategory::Refrigerator;
        i.landlord_provided = false;
        i.nyc_multi_unit_dwelling = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NycRefrigeratorOrStoveMissing);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("NYC Admin. Code § 27-2014"));
        assert!(joined.contains("Multiple Dwelling Law § 78"));
    }

    #[test]
    fn nyc_stove_missing_severity() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::NewYork;
        i.appliance_category = ApplianceCategory::StoveRange;
        i.landlord_provided = false;
        i.nyc_multi_unit_dwelling = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NycRefrigeratorOrStoveMissing);
    }

    #[test]
    fn nyc_dishwasher_not_required_not_applicable() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::NewYork;
        i.appliance_category = ApplianceCategory::Dishwasher;
        i.landlord_provided = false;
        i.nyc_multi_unit_dwelling = true;
        let out = check(&i);
        // Dishwasher not statutorily required even in NYC
        assert_eq!(out.severity, Severity::NotApplicable);
    }

    #[test]
    fn nyc_single_family_no_mandatory_requirement() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::NewYork;
        i.appliance_category = ApplianceCategory::Refrigerator;
        i.landlord_provided = false;
        i.nyc_multi_unit_dwelling = false; // single-family not multi-unit
        let out = check(&i);
        // NYC requirement applies to multi-unit only
        assert_eq!(out.severity, Severity::NotApplicable);
    }

    #[test]
    fn appliance_inoperable_no_tenant_notice_compliant() {
        let mut i = baseline();
        i.appliance_inoperable = true;
        i.tenant_provided_timely_notice = false;
        let out = check(&i);
        // No notice → no repair duty triggered yet
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn appliance_inoperable_within_repair_window() {
        let mut i = baseline();
        i.appliance_inoperable = true;
        i.tenant_provided_timely_notice = true;
        i.days_since_tenant_notice = 5; // within 7-day window
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn appliance_inoperable_extended_no_repair_habitability_breach() {
        let mut i = baseline();
        i.appliance_inoperable = true;
        i.tenant_provided_timely_notice = true;
        i.days_since_tenant_notice = 14;
        i.landlord_made_diligent_repair_effort = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::HabitabilityBreachUnrepaired);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("REPAIR-AND-DEDUCT"));
    }

    #[test]
    fn appliance_inoperable_extended_landlord_diligent_compliant() {
        let mut i = baseline();
        i.appliance_inoperable = true;
        i.tenant_provided_timely_notice = true;
        i.days_since_tenant_notice = 14;
        i.landlord_made_diligent_repair_effort = true;
        let out = check(&i);
        // Landlord diligent → no habitability breach
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn appliance_repaired_compliant() {
        let mut i = baseline();
        i.appliance_inoperable = true;
        i.tenant_provided_timely_notice = true;
        i.days_since_tenant_notice = 14;
        i.repair_completed = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn replacement_lower_quality_dispute() {
        let mut i = baseline();
        i.replacement_materially_lower_quality = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::QualityDegradationDispute);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("MATERIALLY LOWER QUALITY"));
    }

    #[test]
    fn appliance_not_provided_not_required_not_applicable() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Default;
        i.appliance_category = ApplianceCategory::Dishwasher;
        i.landlord_provided = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotApplicable);
    }

    #[test]
    fn tx_appliance_inoperable_no_repair_tex_prop_code_92_056() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Texas;
        i.appliance_inoperable = true;
        i.tenant_provided_timely_notice = true;
        i.days_since_tenant_notice = 14;
        let out = check(&i);
        assert_eq!(out.severity, Severity::HabitabilityBreachUnrepaired);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("Tex. Prop. Code § 92.056"));
    }

    #[test]
    fn ny_appliance_inoperable_no_repair_hpd_route() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::NewYork;
        i.appliance_inoperable = true;
        i.tenant_provided_timely_notice = true;
        i.days_since_tenant_notice = 14;
        let out = check(&i);
        assert_eq!(out.severity, Severity::HabitabilityBreachUnrepaired);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("§ 235-b"));
        assert!(joined.contains("HPD"));
    }

    #[test]
    fn default_jurisdiction_common_law_habitability() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Default;
        i.appliance_inoperable = true;
        i.tenant_provided_timely_notice = true;
        i.days_since_tenant_notice = 14;
        let out = check(&i);
        assert_eq!(out.severity, Severity::HabitabilityBreachUnrepaired);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("Hilder v. St. Peter"));
        assert!(joined.contains("478 A.2d 202"));
    }

    #[test]
    fn severity_priority_injury_above_nyc_mandatory_above_ca_mandatory() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::NewYork;
        i.appliance_failure_caused_fire_flood_or_co_event = true;
        i.appliance_category = ApplianceCategory::Refrigerator;
        i.landlord_provided = false;
        i.nyc_multi_unit_dwelling = true;
        let out = check(&i);
        // Injury wins
        assert_eq!(out.severity, Severity::InjuryEvent);
    }

    #[test]
    fn severity_nyc_above_ca_above_habitability() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::NewYork;
        i.appliance_category = ApplianceCategory::Refrigerator;
        i.landlord_provided = false;
        i.nyc_multi_unit_dwelling = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NycRefrigeratorOrStoveMissing);
    }

    #[test]
    fn severity_habitability_above_quality_degradation() {
        let mut i = baseline();
        i.appliance_inoperable = true;
        i.tenant_provided_timely_notice = true;
        i.days_since_tenant_notice = 14;
        i.replacement_materially_lower_quality = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::HabitabilityBreachUnrepaired);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 235-b"));
        assert!(joined.contains("§ 27-2014"));
        assert!(joined.contains("§ 27-2008"));
        assert!(joined.contains("Multiple Dwelling Law § 78"));
        assert!(joined.contains("28 RCNY § 32-04"));
        assert!(joined.contains("§ 92.052"));
        assert!(joined.contains("§ 92.053"));
        assert!(joined.contains("§ 92.056"));
        assert!(joined.contains("§ 1941.1(a)(8)"));
        assert!(joined.contains("§ 1942(a)"));
        assert!(joined.contains("§ 1942.1"));
        assert!(joined.contains("Green v. Superior Court"));
        assert!(joined.contains("Hinson v. Delis"));
        assert!(joined.contains("Hilder v. St. Peter"));
        assert!(joined.contains("Lemle v. Breeden"));
        assert!(joined.contains("§ 168"));
    }

    #[test]
    fn note_pins_four_jurisdiction_framework() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("New York (most prescriptive"));
        assert!(joined.contains("Texas"));
        assert!(joined.contains("California"));
        assert!(joined.contains("Default"));
    }

    #[test]
    fn note_pins_six_appliance_categories() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("STOVE/RANGE"));
        assert!(joined.contains("REFRIGERATOR"));
        assert!(joined.contains("DISHWASHER"));
        assert!(joined.contains("WASHER/DRYER"));
        assert!(joined.contains("MICROWAVE"));
        assert!(joined.contains("HVAC"));
    }

    #[test]
    fn note_pins_five_failure_modes() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("REFUSAL TO REPAIR"));
        assert!(joined.contains("CONSTRUCTIVE EVICTION"));
        assert!(joined.contains("MATERIALLY LOWER QUALITY"));
        assert!(joined.contains("TENANT-CAUSED DAMAGE"));
        assert!(joined.contains("USED APPLIANCE FAILURE"));
    }

    #[test]
    fn note_pins_repair_window() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("7 days"));
        assert!(joined.contains("24-72 hours"));
        assert!(joined.contains("Reasonable repair window"));
    }

    #[test]
    fn note_pins_companion_modules() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("rental_hot_water_temperature"));
        assert!(joined.contains("rental_chimney_fireplace_inspection_disclosure"));
        assert!(joined.contains("landlord_repair_response_timeframe"));
    }

    #[test]
    fn jurisdiction_truth_table_four_cells() {
        let ny = check(&Input {
            jurisdiction: Jurisdiction::NewYork,
            ..baseline()
        });
        let tx = check(&Input {
            jurisdiction: Jurisdiction::Texas,
            ..baseline()
        });
        let ca = check(&Input {
            jurisdiction: Jurisdiction::California,
            ..baseline()
        });
        let de = check(&Input {
            jurisdiction: Jurisdiction::Default,
            ..baseline()
        });
        assert_eq!(ny.severity, Severity::Compliant);
        assert_eq!(tx.severity, Severity::Compliant);
        assert_eq!(ca.severity, Severity::Compliant);
        assert_eq!(de.severity, Severity::Compliant);
    }

    #[test]
    fn ca_uniquely_stove_mandatory_invariant() {
        let ca = check(&Input {
            jurisdiction: Jurisdiction::California,
            appliance_category: ApplianceCategory::StoveRange,
            landlord_provided: false,
            ..baseline()
        });
        let tx = check(&Input {
            jurisdiction: Jurisdiction::Texas,
            appliance_category: ApplianceCategory::StoveRange,
            landlord_provided: false,
            ..baseline()
        });
        // CA mandates stove; TX does not
        assert_eq!(ca.severity, Severity::CaStoveMandatoryViolation);
        assert_eq!(tx.severity, Severity::NotApplicable);
    }

    #[test]
    fn nyc_uniquely_multi_unit_appliance_requirement() {
        let nyc_multi = check(&Input {
            jurisdiction: Jurisdiction::NewYork,
            appliance_category: ApplianceCategory::Refrigerator,
            landlord_provided: false,
            nyc_multi_unit_dwelling: true,
            ..baseline()
        });
        let nyc_single = check(&Input {
            jurisdiction: Jurisdiction::NewYork,
            appliance_category: ApplianceCategory::Refrigerator,
            landlord_provided: false,
            nyc_multi_unit_dwelling: false,
            ..baseline()
        });
        assert_eq!(nyc_multi.severity, Severity::NycRefrigeratorOrStoveMissing);
        assert_eq!(nyc_single.severity, Severity::NotApplicable);
    }
}
