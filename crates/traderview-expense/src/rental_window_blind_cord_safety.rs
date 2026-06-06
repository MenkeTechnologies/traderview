//! Multi-jurisdictional rental property WINDOW BLIND
//! CORD STRANGULATION SAFETY compliance framework. When
//! a landlord rents a unit, what window-covering cord-
//! safety standards apply, what retrofit obligations
//! attach when child residents under age 8 occupy the
//! unit, what product-recall enforcement risks expose
//! landlord, and what failure-mode liabilities apply
//! after a strangulation incident?
//!
//! Distinct from sibling modules: rental_window_guard_
//! installation (fall protection), rental_carbon_monoxide_
//! detector (CO sensor), rental_bedroom_egress_window
//! (structural), rental_swimming_pool_drain_safety (VGB
//! Act), landlord_security_device_obligations (locks).
//!
//! Three-jurisdiction framework:
//!
//! 1. FEDERAL / CPSC (universal floor) — 16 C.F.R. Part
//!    1260 SAFETY STANDARD FOR OPERATING CORDS ON CUSTOM
//!    WINDOW COVERINGS, final rule 87 Fed. Reg. 73118
//!    (November 28, 2022) under Consumer Product Safety
//!    Act, 15 U.S.C. § 2056. Effective date: May 30, 2023
//!    for all CUSTOM window coverings manufactured after
//!    that date. ELIMINATES free-hanging operating cords,
//!    free-hanging tilt cords, and multiple cords into a
//!    cord connector on all made-to-order custom window
//!    coverings.
//!    ANSI/WCMA A100.1-2018 voluntary standard mandated
//!    CORDLESS or inaccessible-cord designs on all STOCK
//!    products sold in stores and online as of May 2022.
//!    ANSI/WCMA A100.1-2022 update effective June 2024
//!    extends similar requirements to custom products.
//!    Targets strangulation risk to children age 8 or
//!    younger; CPSC reports ~120 child fatalities from
//!    window-covering cord strangulation 2002-2017.
//! 2. CALIFORNIA / PROGRESSIVE STATES — Cal. Civ. Code
//!    § 1941.1 implied warranty of sanitary facilities
//!    plus § 1942.4 untenantable conditions framework
//!    treats child-accessible window-covering cords in a
//!    unit with child residents as an actionable
//!    habitability defect; California Building Code
//!    Title 24 + CPSC-aligned product safety regulations.
//!    Tenant remedies: rent withholding under § 1942(a)
//!    repair-and-deduct, abatement, constructive
//!    eviction.
//! 3. DEFAULT — Common-law implied warranty of
//!    habitability (Hilder v. St. Peter, 478 A.2d 202
//!    (Vt. 1984); Green v. Superior Court, 10 Cal. 3d
//!    616 (1974)) + tort negligence for child-resident
//!    properties; no state statutory landlord retrofit
//!    mandate for pre-existing corded blinds; product
//!    recall enforcement under 16 C.F.R. § 1115 and
//!    Consumer Product Safety Act 15 U.S.C. § 2068
//!    (prohibited acts).
//!
//! Trader-landlord critical because (1) the 2022 CPSC
//! mandatory standard 16 C.F.R. Part 1260 (effective May
//! 30, 2023) does NOT retroactively require landlords to
//! replace existing corded blinds, BUT (2) implied
//! warranty of habitability cases routinely treat
//! corded blinds in child-occupied units as actionable
//! defects, (3) settlement values in child strangulation
//! cases routinely exceed $1M with wrongful death awards
//! routinely exceeding $5M, (4) CPSC recall enforcement
//! under 15 U.S.C. § 2068 can apply if landlord
//! continues to lease units with recalled corded blinds,
//! (5) the most efficient mitigation is proactive
//! retrofit to cordless or inaccessible-cord designs at
//! tenant turnover.
//!
//! Universal failure-mode liability framework:
//! 1. Corded blinds installed in unit with child resident
//!    age 8 or younger → habitability breach + negligence
//!    + premises liability exposure
//! 2. Tenant request to retrofit ignored → constructive
//!    eviction + retaliation exposure under landlord_
//!    retaliation_damages framework
//! 3. CPSC-recalled product in tenant unit → 15 U.S.C.
//!    § 2068 prohibited-acts enforcement + state
//!    consumer-protection liability
//! 4. Custom window covering installed after May 30, 2023
//!    that violates 16 C.F.R. Part 1260 → federal
//!    enforcement + state attorney general action
//! 5. Strangulation injury during tenancy → tort
//!    negligence + wrongful death + IIED (parallel to
//!    tenant_emotional_distress_damages iter 453)
//!
//! Authority: 16 C.F.R. Part 1260 (effective May 30,
//! 2023); 87 Fed. Reg. 73118 (November 28, 2022) Safety
//! Standard for Operating Cords on Custom Window
//! Coverings; ANSI/WCMA A100.1-2018; ANSI/WCMA A100.1-2022
//! (effective June 2024); Consumer Product Safety Act,
//! 15 U.S.C. § 2056; 15 U.S.C. § 2068 (prohibited acts);
//! 16 C.F.R. § 1115 (recall reporting); Cal. Civ. Code
//! § 1941.1; Cal. Civ. Code § 1942(a); Cal. Civ. Code
//! § 1942.4; Hilder v. St. Peter, 478 A.2d 202 (Vt. 1984);
//! Green v. Superior Court, 10 Cal. 3d 616 (1974).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    Federal,
    California,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowCoveringType {
    Cordless,
    InaccessibleCordOnlyWand,
    AccessibleCordedStock,
    AccessibleCordedCustom,
    PreEffectiveDateExisting,
    None,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub covering_type: WindowCoveringType,
    pub child_resident_age_eight_or_younger_present: bool,
    pub installed_or_replaced_after_may_30_2023: bool,
    pub stock_product_installed_after_may_2022: bool,
    pub tenant_retrofit_request_made: bool,
    pub tenant_retrofit_request_addressed: bool,
    pub product_subject_to_cpsc_recall: bool,
    pub strangulation_incident_reported: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    Compliant,
    HabitabilityRisk,
    FederalViolation,
    RecallEnforcement,
    StrangulationIncident,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub severity: Severity,
    pub jurisdiction_specific_actions: Vec<String>,
    pub notes: Vec<String>,
}

pub type RentalWindowBlindCordSafetyInput = Input;
pub type RentalWindowBlindCordSafetyResult = Output;

pub fn check(input: &Input) -> Output {
    let notes: Vec<String> = vec![
        "Three-jurisdiction framework: Federal/CPSC (16 C.F.R. Part 1260 mandatory standard effective May 30, 2023 for custom window coverings under 87 Fed. Reg. 73118; ANSI/WCMA A100.1-2018 voluntary standard cordless stock since May 2022; ANSI/WCMA A100.1-2022 custom-product extension effective June 2024); California/progressive states (Cal. Civ. Code § 1941.1 implied warranty + § 1942.4 untenantable conditions + § 1942(a) repair-and-deduct); Default (common-law implied warranty of habitability per Hilder v. St. Peter 478 A.2d 202 (Vt. 1984) + tort negligence + CPSC recall enforcement under 15 U.S.C. § 2068).".to_string(),
        "16 C.F.R. Part 1260 eliminates free-hanging operating cords + free-hanging tilt cords + multiple cords into cord connectors on all CUSTOM window coverings manufactured after May 30, 2023; CPSC reports ~120 child fatalities from window-covering cord strangulation 2002-2017 to children age 8 or younger.".to_string(),
        "Federal mandatory standard does NOT retroactively require landlord to replace pre-existing corded blinds installed before May 30, 2023; however implied warranty of habitability cases routinely treat corded blinds in child-occupied units as actionable defects regardless of installation date.".to_string(),
        "Five universal failure-mode liabilities: (1) corded blinds + child resident age 8 or younger = habitability breach + premises liability; (2) ignored tenant retrofit request = constructive eviction + retaliation; (3) CPSC-recalled product in tenant unit = 15 U.S.C. § 2068 prohibited-acts enforcement + state consumer-protection liability; (4) custom blind installed after May 30, 2023 violating 16 C.F.R. Part 1260 = federal enforcement; (5) strangulation injury = tort negligence + wrongful death + IIED parallel to tenant_emotional_distress_damages iter 453.".to_string(),
        "Companion modules: rental_window_guard_installation (window fall protection), rental_carbon_monoxide_detector, rental_bedroom_egress_window, rental_swimming_pool_drain_safety, landlord_security_device_obligations, tenant_emotional_distress_damages, landlord_retaliation_damages.".to_string(),
    ];
    let mut actions: Vec<String> = Vec::new();

    if matches!(input.covering_type, WindowCoveringType::None) {
        let mut n = notes;
        n.push("No window covering present — § 1260 + ANSI/WCMA not applicable; basic habitability still applies.".to_string());
        return Output {
            severity: Severity::NotApplicable,
            jurisdiction_specific_actions: actions,
            notes: n,
        };
    }

    if input.strangulation_incident_reported {
        actions.push("Strangulation incident reported: engage emergency services + counsel; tort negligence + wrongful death + IIED exposure parallel to tenant_emotional_distress_damages iter 453; preserve evidence; notify CPSC under 16 C.F.R. § 1115 if product defect contributed.".to_string());
    }

    if input.product_subject_to_cpsc_recall {
        actions.push("CPSC-recalled product installed: 15 U.S.C. § 2068 prohibited acts to sell or distribute (including via lease) recalled products; 16 C.F.R. § 1115 substantial-product-hazard reporting may apply; immediate retrofit + tenant notice required; coordinate with CPSC recall remedy program; state consumer-protection statutes may apply.".to_string());
    }

    let is_custom_violation = input.installed_or_replaced_after_may_30_2023
        && matches!(
            input.covering_type,
            WindowCoveringType::AccessibleCordedCustom
        );
    if is_custom_violation {
        actions.push("Custom window covering installed after May 30, 2023 with accessible cord — VIOLATES 16 C.F.R. Part 1260 mandatory standard under 87 Fed. Reg. 73118 (Nov 28, 2022); federal CPSC enforcement + state AG action exposure; replace immediately with cordless or inaccessible-cord design.".to_string());
    }

    let is_stock_violation = input.stock_product_installed_after_may_2022
        && matches!(
            input.covering_type,
            WindowCoveringType::AccessibleCordedStock
        );
    if is_stock_violation {
        actions.push("Stock window-covering product installed after May 2022 with accessible cord — violates ANSI/WCMA A100.1-2018 cordless-stock requirement; product likely non-conforming + subject to CPSC recall investigation; replace immediately.".to_string());
    }

    let child_present_with_corded = input.child_resident_age_eight_or_younger_present
        && matches!(
            input.covering_type,
            WindowCoveringType::AccessibleCordedStock
                | WindowCoveringType::AccessibleCordedCustom
                | WindowCoveringType::PreEffectiveDateExisting
        );
    if child_present_with_corded {
        actions.push("Child resident age 8 or younger present with accessible corded blinds: implied warranty of habitability + premises-liability exposure; CPSC strangulation fatality data 2002-2017 (~120 children); recommend retrofit to cordless designs or inaccessible-cord wand at tenant request.".to_string());
    }

    if input.tenant_retrofit_request_made && !input.tenant_retrofit_request_addressed {
        actions.push("Tenant retrofit request unaddressed: constructive eviction + landlord_retaliation_damages exposure; particularly acute if child resident present; retrofit timeline should be days to weeks, not months.".to_string());
    }

    match input.jurisdiction {
        Jurisdiction::Federal => {}
        Jurisdiction::California => {
            actions.push("California: Cal. Civ. Code § 1941.1 implied warranty of sanitary facilities + § 1942.4 untenantable conditions framework — corded blinds in child-occupied unit may trigger tenant rent-withholding under § 1942(a) repair-and-deduct or abatement.".to_string());
        }
        Jurisdiction::Default => {
            actions.push("Default jurisdiction: common-law implied warranty of habitability per Hilder v. St. Peter, 478 A.2d 202 (Vt. 1984) + Green v. Superior Court, 10 Cal. 3d 616 (1974); tort negligence + premises liability for child-resident properties.".to_string());
        }
    }

    let severity = if input.strangulation_incident_reported {
        Severity::StrangulationIncident
    } else if input.product_subject_to_cpsc_recall {
        Severity::RecallEnforcement
    } else if is_custom_violation || is_stock_violation {
        Severity::FederalViolation
    } else if child_present_with_corded {
        Severity::HabitabilityRisk
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
            jurisdiction: Jurisdiction::Federal,
            covering_type: WindowCoveringType::Cordless,
            child_resident_age_eight_or_younger_present: false,
            installed_or_replaced_after_may_30_2023: false,
            stock_product_installed_after_may_2022: false,
            tenant_retrofit_request_made: false,
            tenant_retrofit_request_addressed: false,
            product_subject_to_cpsc_recall: false,
            strangulation_incident_reported: false,
        }
    }

    #[test]
    fn no_window_covering_not_applicable() {
        let mut i = baseline();
        i.covering_type = WindowCoveringType::None;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotApplicable);
    }

    #[test]
    fn cordless_compliant_baseline() {
        let i = baseline();
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn inaccessible_cord_wand_compliant() {
        let mut i = baseline();
        i.covering_type = WindowCoveringType::InaccessibleCordOnlyWand;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn pre_effective_date_existing_no_child_no_violation() {
        let mut i = baseline();
        i.covering_type = WindowCoveringType::PreEffectiveDateExisting;
        let out = check(&i);
        // No child present + pre-effective-date existing → no federal violation; compliant
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn pre_effective_date_with_child_habitability_risk() {
        let mut i = baseline();
        i.covering_type = WindowCoveringType::PreEffectiveDateExisting;
        i.child_resident_age_eight_or_younger_present = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::HabitabilityRisk);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("Child resident age 8 or younger"));
        assert!(joined.contains("CPSC strangulation fatality data"));
    }

    #[test]
    fn custom_corded_installed_after_may_30_2023_federal_violation() {
        let mut i = baseline();
        i.covering_type = WindowCoveringType::AccessibleCordedCustom;
        i.installed_or_replaced_after_may_30_2023 = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::FederalViolation);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("16 C.F.R. Part 1260"));
        assert!(joined.contains("May 30, 2023"));
        assert!(joined.contains("87 Fed. Reg. 73118"));
    }

    #[test]
    fn custom_corded_installed_before_may_30_2023_no_federal_violation() {
        let mut i = baseline();
        i.covering_type = WindowCoveringType::AccessibleCordedCustom;
        i.installed_or_replaced_after_may_30_2023 = false;
        let out = check(&i);
        // No federal violation; compliant on federal basis (no child present)
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn stock_corded_installed_after_may_2022_federal_violation() {
        let mut i = baseline();
        i.covering_type = WindowCoveringType::AccessibleCordedStock;
        i.stock_product_installed_after_may_2022 = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::FederalViolation);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("ANSI/WCMA A100.1-2018"));
    }

    #[test]
    fn cpsc_recall_product_recall_enforcement_severity() {
        let mut i = baseline();
        i.covering_type = WindowCoveringType::AccessibleCordedStock;
        i.product_subject_to_cpsc_recall = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::RecallEnforcement);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("15 U.S.C. § 2068"));
        assert!(joined.contains("16 C.F.R. § 1115"));
    }

    #[test]
    fn strangulation_incident_top_severity() {
        let mut i = baseline();
        i.covering_type = WindowCoveringType::PreEffectiveDateExisting;
        i.strangulation_incident_reported = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::StrangulationIncident);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("Strangulation incident"));
        assert!(joined.contains("tenant_emotional_distress_damages"));
    }

    #[test]
    fn tenant_retrofit_request_unaddressed_constructive_eviction_note() {
        let mut i = baseline();
        i.covering_type = WindowCoveringType::PreEffectiveDateExisting;
        i.tenant_retrofit_request_made = true;
        i.tenant_retrofit_request_addressed = false;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("constructive eviction"));
        assert!(joined.contains("landlord_retaliation_damages"));
    }

    #[test]
    fn tenant_retrofit_request_addressed_no_warning() {
        let mut i = baseline();
        i.covering_type = WindowCoveringType::Cordless;
        i.tenant_retrofit_request_made = true;
        i.tenant_retrofit_request_addressed = true;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(!joined.contains("Tenant retrofit request unaddressed"));
    }

    #[test]
    fn california_cites_1941_1_and_1942() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::California;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("§ 1941.1"));
        assert!(joined.contains("§ 1942.4"));
        assert!(joined.contains("§ 1942(a)"));
    }

    #[test]
    fn default_jurisdiction_common_law_habitability() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Default;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("Hilder v. St. Peter"));
        assert!(joined.contains("478 A.2d 202"));
        assert!(joined.contains("Green v. Superior Court"));
        assert!(joined.contains("10 Cal. 3d 616"));
    }

    #[test]
    fn severity_priority_incident_above_recall_above_federal_above_habitability() {
        // Strangulation wins everything
        let mut i = baseline();
        i.covering_type = WindowCoveringType::AccessibleCordedCustom;
        i.installed_or_replaced_after_may_30_2023 = true;
        i.product_subject_to_cpsc_recall = true;
        i.strangulation_incident_reported = true;
        i.child_resident_age_eight_or_younger_present = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::StrangulationIncident);
    }

    #[test]
    fn severity_recall_above_federal_above_habitability() {
        let mut i = baseline();
        i.covering_type = WindowCoveringType::AccessibleCordedCustom;
        i.installed_or_replaced_after_may_30_2023 = true;
        i.product_subject_to_cpsc_recall = true;
        i.child_resident_age_eight_or_younger_present = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::RecallEnforcement);
    }

    #[test]
    fn severity_federal_above_habitability() {
        let mut i = baseline();
        i.covering_type = WindowCoveringType::AccessibleCordedCustom;
        i.installed_or_replaced_after_may_30_2023 = true;
        i.child_resident_age_eight_or_younger_present = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::FederalViolation);
    }

    #[test]
    fn child_age_8_with_accessible_corded_stock_habitability_risk() {
        let mut i = baseline();
        i.covering_type = WindowCoveringType::AccessibleCordedStock;
        i.child_resident_age_eight_or_younger_present = true;
        let out = check(&i);
        // No federal violation flag set; child + corded = habitability risk
        assert_eq!(out.severity, Severity::HabitabilityRisk);
    }

    #[test]
    fn no_child_pre_effective_date_existing_compliant() {
        let mut i = baseline();
        i.covering_type = WindowCoveringType::PreEffectiveDateExisting;
        i.child_resident_age_eight_or_younger_present = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn cordless_with_child_present_still_compliant() {
        let mut i = baseline();
        i.covering_type = WindowCoveringType::Cordless;
        i.child_resident_age_eight_or_younger_present = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn inaccessible_cord_wand_with_child_compliant() {
        let mut i = baseline();
        i.covering_type = WindowCoveringType::InaccessibleCordOnlyWand;
        i.child_resident_age_eight_or_younger_present = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("16 C.F.R. Part 1260"));
        assert!(joined.contains("May 30, 2023"));
        assert!(joined.contains("87 Fed. Reg. 73118"));
        assert!(joined.contains("ANSI/WCMA A100.1-2018"));
        assert!(joined.contains("ANSI/WCMA A100.1-2022"));
        assert!(joined.contains("June 2024"));
        assert!(joined.contains("15 U.S.C. § 2068"));
        assert!(joined.contains("§ 1941.1"));
        assert!(joined.contains("§ 1942.4"));
        assert!(joined.contains("§ 1942(a)"));
        assert!(joined.contains("Hilder v. St. Peter"));
        assert!(joined.contains("478 A.2d 202"));
    }

    #[test]
    fn note_pins_three_jurisdiction_framework() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("Federal/CPSC"));
        assert!(joined.contains("California/progressive"));
        assert!(joined.contains("Default"));
    }

    #[test]
    fn note_pins_five_failure_modes() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("corded blinds + child"));
        assert!(joined.contains("retrofit request"));
        assert!(joined.contains("recalled product"));
        assert!(joined.contains("after May 30, 2023"));
        assert!(joined.contains("strangulation injury"));
    }

    #[test]
    fn note_pins_no_retroactive_replacement_requirement() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("does NOT retroactively"));
        assert!(joined.contains("pre-existing corded blinds"));
    }

    #[test]
    fn note_pins_cpsc_120_fatality_data() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("~120 child fatalities"));
        assert!(joined.contains("2002-2017"));
        assert!(joined.contains("age 8 or younger"));
    }

    #[test]
    fn note_pins_companion_modules() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("rental_window_guard_installation"));
        assert!(joined.contains("rental_carbon_monoxide_detector"));
        assert!(joined.contains("tenant_emotional_distress_damages"));
        assert!(joined.contains("landlord_retaliation_damages"));
    }

    #[test]
    fn covering_type_truth_table() {
        // Cordless = Compliant
        let a = check(&Input {
            covering_type: WindowCoveringType::Cordless,
            child_resident_age_eight_or_younger_present: true,
            ..baseline()
        });
        assert_eq!(a.severity, Severity::Compliant);
        // InaccessibleCordOnlyWand = Compliant
        let b = check(&Input {
            covering_type: WindowCoveringType::InaccessibleCordOnlyWand,
            child_resident_age_eight_or_younger_present: true,
            ..baseline()
        });
        assert_eq!(b.severity, Severity::Compliant);
        // AccessibleCordedStock with child = HabitabilityRisk
        let c = check(&Input {
            covering_type: WindowCoveringType::AccessibleCordedStock,
            child_resident_age_eight_or_younger_present: true,
            ..baseline()
        });
        assert_eq!(c.severity, Severity::HabitabilityRisk);
        // PreEffectiveDateExisting without child = Compliant
        let d = check(&Input {
            covering_type: WindowCoveringType::PreEffectiveDateExisting,
            child_resident_age_eight_or_younger_present: false,
            ..baseline()
        });
        assert_eq!(d.severity, Severity::Compliant);
    }

    #[test]
    fn ca_uniquely_cites_repair_and_deduct() {
        let ca = check(&Input {
            jurisdiction: Jurisdiction::California,
            ..baseline()
        });
        let de = check(&Input {
            jurisdiction: Jurisdiction::Default,
            ..baseline()
        });
        let joined_ca = ca.jurisdiction_specific_actions.join(" ");
        let joined_de = de.jurisdiction_specific_actions.join(" ");
        assert!(joined_ca.contains("repair-and-deduct"));
        assert!(!joined_de.contains("repair-and-deduct"));
    }

    #[test]
    fn custom_corded_installed_after_may_30_2023_pinpoints_part_1260() {
        let mut i = baseline();
        i.covering_type = WindowCoveringType::AccessibleCordedCustom;
        i.installed_or_replaced_after_may_30_2023 = true;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("Part 1260"));
        assert!(joined.contains("mandatory standard"));
    }

    #[test]
    fn recall_enforcement_pins_2068_and_1115() {
        let mut i = baseline();
        i.covering_type = WindowCoveringType::AccessibleCordedStock;
        i.product_subject_to_cpsc_recall = true;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("§ 2068"));
        assert!(joined.contains("recall remedy program"));
        assert!(joined.contains("state consumer-protection"));
    }

    #[test]
    fn strangulation_incident_notes_cpsc_1115_reporting() {
        let mut i = baseline();
        i.strangulation_incident_reported = true;
        i.covering_type = WindowCoveringType::PreEffectiveDateExisting;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("CPSC under 16 C.F.R. § 1115"));
        assert!(joined.contains("preserve evidence"));
    }
}
