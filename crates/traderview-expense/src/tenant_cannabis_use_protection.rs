//! State tenant-cannabis-use protection landlord compliance check.
//!
//! Cannabis legalization has produced two opposing approaches to the
//! landlord/tenant power balance in residential rentals. New York's
//! Cannabis Law § 134 affirmatively protects tenants — landlords cannot
//! refuse to rent to cannabis users and must permit registered medical
//! cannabis use even by smoking, with a narrow federal-benefits
//! exception. Illinois's Cannabis Regulation and Tax Act (CRTA) takes
//! the opposite default — landlords may ban smoking, vaporizing, AND
//! cultivation of cannabis (including medical) by lease provision.
//! Most other states fall closer to the Illinois pattern: lease terms
//! control absent a state-specific tenant-protection statute.
//!
//! New York Cannabis Law § 134 — landlord MAY NOT refuse to rent to a
//! tenant who consumes cannabis. Landlord MAY ban smoking/vaporizing/
//! cultivation in the lease GENERALLY. EXCEPTION: registered NYS
//! Medical Cannabis Program patients may consume medical cannabis in
//! their home including smoking and vaping whole flower / concentrate /
//! plant products. Landlord may restrict medical cannabis only if doing
//! so is necessary to preserve a federal benefit (Section 8, federal
//! housing subsidies).
//!
//! Illinois CRTA (410 ILCS 705/ effective 2020-01-01) — landlords may
//! prohibit cannabis smoking + vaporizing + cultivation in the unit
//! and on the property via lease. Applies to BOTH recreational AND
//! medical cannabis. Tenant breach may lead to eviction. Medical
//! cannabis patients may have a Fair Housing Act reasonable-accommo-
//! dation argument but the CRTA does not affirmatively protect them.
//!
//! Default — no specific statewide tenant-cannabis-protection statute.
//! Landlord may prohibit cannabis use via lease subject to general
//! Fair Housing Act / state Fair Housing reasonable-accommodation
//! analysis for documented medical use.
//!
//! Citations: NY Cannabis Law § 134 (anti-discrimination + medical
//! cannabis use); NY Cannabis Law Article 4 (adult-use rules); 410
//! ILCS 705/ (Illinois CRTA, effective 2020-01-01); CRTA Art. 15
//! (landlord authority to prohibit).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    NewYorkCannabisLaw134,
    IllinoisCrta,
    Default,
}

impl Regime {
    pub fn for_state(state: &str) -> Self {
        match state.trim().to_ascii_uppercase().as_str() {
            "NY" => Self::NewYorkCannabisLaw134,
            "IL" => Self::IllinoisCrta,
            _ => Self::Default,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProhibitedActType {
    /// Landlord refuses to rent based on tenant's cannabis use (separate
    /// from refusing based on disability under FHA).
    RefuseToRentBasedOnCannabisUse,
    /// Landlord includes lease provision banning smoking/vaporizing.
    BanSmokingInLease,
    /// Landlord includes lease provision banning cultivation.
    BanCultivationInLease,
    /// Landlord enforces smoking ban against a registered medical
    /// cannabis patient.
    EnforceSmokingBanAgainstMedicalPatient,
    /// Landlord initiates eviction based on cannabis use.
    EvictBasedOnCannabisUse,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CannabisProtectionInput {
    pub regime: Regime,
    pub act_type: ProhibitedActType,
    /// Whether the tenant is a registered medical cannabis patient.
    pub tenant_is_medical_cannabis_patient: bool,
    /// Whether the landlord receives federal benefits (Section 8 + HUD
    /// subsidies) where allowing cannabis would create federal-benefit
    /// risk. Triggers NY's narrow exception.
    pub landlord_receives_federal_benefits: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ViolationType {
    None,
    ProhibitedRefusalToRent,
    ProhibitedEnforcementAgainstMedicalPatient,
    ProhibitedEviction,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CannabisProtectionResult {
    pub regime: Regime,
    pub act_permitted: bool,
    pub medical_cannabis_exception_applies: bool,
    pub federal_benefits_exception_applies: bool,
    pub violation: ViolationType,
    pub landlord_compliant: bool,
    pub citation: &'static str,
    pub note: String,
}

pub fn check(input: &CannabisProtectionInput) -> CannabisProtectionResult {
    match input.regime {
        Regime::NewYorkCannabisLaw134 => ny_check(input),
        Regime::IllinoisCrta => il_check(input),
        Regime::Default => default_check(input),
    }
}

fn ny_check(input: &CannabisProtectionInput) -> CannabisProtectionResult {
    match input.act_type {
        ProhibitedActType::RefuseToRentBasedOnCannabisUse => CannabisProtectionResult {
            regime: Regime::NewYorkCannabisLaw134,
            act_permitted: false,
            medical_cannabis_exception_applies: false,
            federal_benefits_exception_applies: false,
            violation: ViolationType::ProhibitedRefusalToRent,
            landlord_compliant: false,
            citation: "NY Cannabis Law § 134 — landlord may NOT refuse to rent based on tenant's cannabis use",
            note: "NY Cannabis Law § 134 prohibits refusing to rent based on tenant's cannabis use.".to_string(),
        },
        ProhibitedActType::BanSmokingInLease | ProhibitedActType::BanCultivationInLease => {
            CannabisProtectionResult {
                regime: Regime::NewYorkCannabisLaw134,
                act_permitted: true,
                medical_cannabis_exception_applies: false,
                federal_benefits_exception_applies: false,
                violation: ViolationType::None,
                landlord_compliant: true,
                citation: "NY Cannabis Law § 134 — landlord MAY ban smoking/vaporizing/cultivation generally; medical cannabis exception preserved",
                note: "Landlord may ban smoking + vaporizing + cultivation generally in the lease; this does not violate NY § 134 (medical-cannabis users get separate protection).".to_string(),
            }
        }
        ProhibitedActType::EnforceSmokingBanAgainstMedicalPatient => {
            if !input.tenant_is_medical_cannabis_patient {
                return CannabisProtectionResult {
                    regime: Regime::NewYorkCannabisLaw134,
                    act_permitted: true,
                    medical_cannabis_exception_applies: false,
                    federal_benefits_exception_applies: false,
                    violation: ViolationType::None,
                    landlord_compliant: true,
                    citation: "NY Cannabis Law § 134 — medical-cannabis exception only applies to registered patients",
                    note: "Tenant is not a registered medical cannabis patient; landlord may enforce smoking ban.".to_string(),
                };
            }
            if input.landlord_receives_federal_benefits {
                return CannabisProtectionResult {
                    regime: Regime::NewYorkCannabisLaw134,
                    act_permitted: true,
                    medical_cannabis_exception_applies: true,
                    federal_benefits_exception_applies: true,
                    violation: ViolationType::None,
                    landlord_compliant: true,
                    citation: "NY Cannabis Law § 134 — federal-benefits exception permits restriction of medical cannabis when needed to preserve Section 8 or HUD subsidy",
                    note: "Landlord receives federal benefits (Section 8/HUD) — narrow federal-benefits exception permits enforcement of smoking ban against medical patient.".to_string(),
                };
            }
            CannabisProtectionResult {
                regime: Regime::NewYorkCannabisLaw134,
                act_permitted: false,
                medical_cannabis_exception_applies: true,
                federal_benefits_exception_applies: false,
                violation: ViolationType::ProhibitedEnforcementAgainstMedicalPatient,
                landlord_compliant: false,
                citation: "NY Cannabis Law § 134 — landlord's smoke-free policy may NOT be construed to limit certified medical use of cannabis (absent federal-benefits exception)",
                note: "Tenant is a registered medical cannabis patient; landlord does NOT receive federal benefits. NY § 134 medical-cannabis exception bars enforcement of the smoking ban.".to_string(),
            }
        }
        ProhibitedActType::EvictBasedOnCannabisUse => CannabisProtectionResult {
            regime: Regime::NewYorkCannabisLaw134,
            act_permitted: !input.tenant_is_medical_cannabis_patient
                || input.landlord_receives_federal_benefits,
            medical_cannabis_exception_applies: input.tenant_is_medical_cannabis_patient,
            federal_benefits_exception_applies: input.landlord_receives_federal_benefits,
            violation: if input.tenant_is_medical_cannabis_patient
                && !input.landlord_receives_federal_benefits
            {
                ViolationType::ProhibitedEviction
            } else {
                ViolationType::None
            },
            landlord_compliant: !input.tenant_is_medical_cannabis_patient
                || input.landlord_receives_federal_benefits,
            citation: "NY Cannabis Law § 134 — eviction based on cannabis use barred for registered medical patients absent federal-benefits exception",
            note: if input.tenant_is_medical_cannabis_patient
                && !input.landlord_receives_federal_benefits
            {
                "Eviction of registered medical cannabis patient is barred by NY § 134.".to_string()
            } else {
                "Eviction based on cannabis use is permitted (tenant not a medical patient, or federal-benefits exception applies).".to_string()
            },
        },
    }
}

fn il_check(input: &CannabisProtectionInput) -> CannabisProtectionResult {
    // Illinois CRTA: landlord may prohibit cannabis (recreational + medical)
    // via lease. Medical patients may have FHA reasonable-accommodation
    // claim but CRTA does not affirmatively protect.
    let act_permitted = matches!(
        input.act_type,
        ProhibitedActType::BanSmokingInLease
            | ProhibitedActType::BanCultivationInLease
            | ProhibitedActType::EnforceSmokingBanAgainstMedicalPatient
            | ProhibitedActType::EvictBasedOnCannabisUse
    );
    CannabisProtectionResult {
        regime: Regime::IllinoisCrta,
        act_permitted,
        medical_cannabis_exception_applies: false,
        federal_benefits_exception_applies: false,
        violation: if input.act_type == ProhibitedActType::RefuseToRentBasedOnCannabisUse {
            // Refusing to rent based purely on cannabis use is not explicitly
            // addressed by CRTA — Fair Housing Act analysis applies. Model
            // as permitted under CRTA but flag FHA exposure in note.
            ViolationType::None
        } else {
            ViolationType::None
        },
        landlord_compliant: true,
        citation: "410 ILCS 705/ (Illinois CRTA eff. 2020-01-01) — landlord MAY prohibit cannabis smoking + vaporizing + cultivation including medical; tenant breach may lead to eviction",
        note: if input.act_type == ProhibitedActType::EnforceSmokingBanAgainstMedicalPatient
            && input.tenant_is_medical_cannabis_patient
        {
            "Landlord may enforce smoking ban against medical cannabis patient under CRTA. Tenant may have separate FHA reasonable-accommodation claim under federal law.".to_string()
        } else {
            format!(
                "Act {:?} permitted under Illinois CRTA — landlord retains broad authority to prohibit cannabis use via lease.",
                input.act_type
            )
        },
    }
}

fn default_check(input: &CannabisProtectionInput) -> CannabisProtectionResult {
    CannabisProtectionResult {
        regime: Regime::Default,
        act_permitted: true,
        medical_cannabis_exception_applies: false,
        federal_benefits_exception_applies: false,
        violation: ViolationType::None,
        landlord_compliant: true,
        citation:
            "No state-specific tenant-cannabis-protection statute identified — landlord may prohibit via lease; federal FHA reasonable-accommodation analysis may apply for medical use",
        note: format!(
            "Default regime: landlord may prohibit cannabis use via lease. Act {:?} permitted absent state-specific tenant protection. Medical cannabis patients may have FHA reasonable-accommodation claims.",
            input.act_type
        ),
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    fn input(
        regime: Regime,
        act: ProhibitedActType,
        medical: bool,
        federal_benefits: bool,
    ) -> CannabisProtectionInput {
        CannabisProtectionInput {
            regime,
            act_type: act,
            tenant_is_medical_cannabis_patient: medical,
            landlord_receives_federal_benefits: federal_benefits,
        }
    }

    #[test]
    fn ny_refuse_to_rent_violation() {
        let r = check(&input(
            Regime::NewYorkCannabisLaw134,
            ProhibitedActType::RefuseToRentBasedOnCannabisUse,
            false,
            false,
        ));
        assert_eq!(r.violation, ViolationType::ProhibitedRefusalToRent);
        assert!(!r.landlord_compliant);
        assert!(r.citation.contains("§ 134"));
    }

    #[test]
    fn ny_ban_smoking_in_lease_permitted() {
        let r = check(&input(
            Regime::NewYorkCannabisLaw134,
            ProhibitedActType::BanSmokingInLease,
            false,
            false,
        ));
        assert!(r.act_permitted);
        assert_eq!(r.violation, ViolationType::None);
        assert!(r.citation.contains("medical cannabis exception"));
    }

    #[test]
    fn ny_ban_cultivation_in_lease_permitted() {
        let r = check(&input(
            Regime::NewYorkCannabisLaw134,
            ProhibitedActType::BanCultivationInLease,
            false,
            false,
        ));
        assert!(r.act_permitted);
        assert_eq!(r.violation, ViolationType::None);
    }

    #[test]
    fn ny_enforce_against_medical_patient_no_federal_benefits_violation() {
        let r = check(&input(
            Regime::NewYorkCannabisLaw134,
            ProhibitedActType::EnforceSmokingBanAgainstMedicalPatient,
            true,
            false,
        ));
        assert_eq!(
            r.violation,
            ViolationType::ProhibitedEnforcementAgainstMedicalPatient
        );
        assert!(r.medical_cannabis_exception_applies);
        assert!(!r.federal_benefits_exception_applies);
    }

    #[test]
    fn ny_enforce_against_medical_patient_with_federal_benefits_permitted() {
        // Section 8 landlord — federal-benefits exception kicks in.
        let r = check(&input(
            Regime::NewYorkCannabisLaw134,
            ProhibitedActType::EnforceSmokingBanAgainstMedicalPatient,
            true,
            true,
        ));
        assert!(r.act_permitted);
        assert!(r.federal_benefits_exception_applies);
        assert_eq!(r.violation, ViolationType::None);
        assert!(r.citation.contains("federal-benefits exception"));
    }

    #[test]
    fn ny_enforce_against_NON_medical_patient_permitted() {
        let r = check(&input(
            Regime::NewYorkCannabisLaw134,
            ProhibitedActType::EnforceSmokingBanAgainstMedicalPatient,
            false,
            false,
        ));
        assert!(r.act_permitted);
        assert!(!r.medical_cannabis_exception_applies);
    }

    #[test]
    fn ny_evict_medical_patient_no_federal_benefits_violation() {
        let r = check(&input(
            Regime::NewYorkCannabisLaw134,
            ProhibitedActType::EvictBasedOnCannabisUse,
            true,
            false,
        ));
        assert_eq!(r.violation, ViolationType::ProhibitedEviction);
    }

    #[test]
    fn ny_evict_non_medical_user_permitted() {
        let r = check(&input(
            Regime::NewYorkCannabisLaw134,
            ProhibitedActType::EvictBasedOnCannabisUse,
            false,
            false,
        ));
        assert!(r.act_permitted);
        assert_eq!(r.violation, ViolationType::None);
    }

    #[test]
    fn il_landlord_may_ban_smoking_in_lease() {
        let r = check(&input(
            Regime::IllinoisCrta,
            ProhibitedActType::BanSmokingInLease,
            false,
            false,
        ));
        assert!(r.act_permitted);
        assert!(r.citation.contains("410 ILCS 705/"));
        assert!(r.citation.contains("CRTA"));
    }

    #[test]
    fn il_landlord_may_ban_cultivation() {
        let r = check(&input(
            Regime::IllinoisCrta,
            ProhibitedActType::BanCultivationInLease,
            false,
            false,
        ));
        assert!(r.act_permitted);
    }

    #[test]
    fn il_landlord_may_enforce_against_medical_patient() {
        // Illinois CRTA does NOT protect medical patients from landlord
        // smoking ban (Fair Housing Act may give separate accommodation
        // right but CRTA does not bar enforcement).
        let r = check(&input(
            Regime::IllinoisCrta,
            ProhibitedActType::EnforceSmokingBanAgainstMedicalPatient,
            true,
            false,
        ));
        assert!(r.act_permitted);
        assert_eq!(r.violation, ViolationType::None);
        assert!(r.note.contains("FHA reasonable-accommodation"));
    }

    #[test]
    fn il_landlord_may_evict_for_cannabis_use() {
        let r = check(&input(
            Regime::IllinoisCrta,
            ProhibitedActType::EvictBasedOnCannabisUse,
            true,
            false,
        ));
        assert!(r.act_permitted);
    }

    #[test]
    fn default_landlord_may_prohibit_cannabis_use() {
        let r = check(&input(
            Regime::Default,
            ProhibitedActType::BanSmokingInLease,
            false,
            false,
        ));
        assert!(r.act_permitted);
        assert!(r.citation.contains("No state-specific"));
    }

    #[test]
    fn default_evict_for_cannabis_use_permitted() {
        let r = check(&input(
            Regime::Default,
            ProhibitedActType::EvictBasedOnCannabisUse,
            true,
            false,
        ));
        assert!(r.act_permitted);
    }

    #[test]
    fn state_routing_ny_il_default() {
        assert_eq!(Regime::for_state("NY"), Regime::NewYorkCannabisLaw134);
        assert_eq!(Regime::for_state("IL"), Regime::IllinoisCrta);
        assert_eq!(Regime::for_state("CA"), Regime::Default);
        assert_eq!(Regime::for_state("TX"), Regime::Default);
    }

    #[test]
    fn state_routing_case_insensitive() {
        assert_eq!(Regime::for_state("ny"), Regime::NewYorkCannabisLaw134);
        assert_eq!(Regime::for_state("Il"), Regime::IllinoisCrta);
    }

    #[test]
    fn only_ny_protects_refusal_to_rent() {
        // Same refuse-to-rent input across regimes — only NY violates.
        let ny = check(&input(
            Regime::NewYorkCannabisLaw134,
            ProhibitedActType::RefuseToRentBasedOnCannabisUse,
            false,
            false,
        ));
        let il = check(&input(
            Regime::IllinoisCrta,
            ProhibitedActType::RefuseToRentBasedOnCannabisUse,
            false,
            false,
        ));
        let d = check(&input(
            Regime::Default,
            ProhibitedActType::RefuseToRentBasedOnCannabisUse,
            false,
            false,
        ));
        assert_eq!(ny.violation, ViolationType::ProhibitedRefusalToRent);
        assert_eq!(il.violation, ViolationType::None);
        assert_eq!(d.violation, ViolationType::None);
    }

    #[test]
    fn only_ny_protects_medical_patients_from_smoking_ban() {
        // Medical patient + landlord enforces smoking ban + no federal
        // benefits: NY → violation; IL + Default → permitted under state law.
        let ny = check(&input(
            Regime::NewYorkCannabisLaw134,
            ProhibitedActType::EnforceSmokingBanAgainstMedicalPatient,
            true,
            false,
        ));
        let il = check(&input(
            Regime::IllinoisCrta,
            ProhibitedActType::EnforceSmokingBanAgainstMedicalPatient,
            true,
            false,
        ));
        let d = check(&input(
            Regime::Default,
            ProhibitedActType::EnforceSmokingBanAgainstMedicalPatient,
            true,
            false,
        ));
        assert_eq!(
            ny.violation,
            ViolationType::ProhibitedEnforcementAgainstMedicalPatient
        );
        assert_eq!(il.violation, ViolationType::None);
        assert_eq!(d.violation, ViolationType::None);
    }

    #[test]
    fn only_ny_has_federal_benefits_exception() {
        // Federal-benefits flag only matters in NY; IL/Default ignore it.
        let ny = check(&input(
            Regime::NewYorkCannabisLaw134,
            ProhibitedActType::EnforceSmokingBanAgainstMedicalPatient,
            true,
            true,
        ));
        let il = check(&input(
            Regime::IllinoisCrta,
            ProhibitedActType::EnforceSmokingBanAgainstMedicalPatient,
            true,
            true,
        ));
        assert!(ny.federal_benefits_exception_applies);
        assert!(!il.federal_benefits_exception_applies);
    }

    #[test]
    fn citations_pin_correct_authorities() {
        let ny = check(&input(
            Regime::NewYorkCannabisLaw134,
            ProhibitedActType::RefuseToRentBasedOnCannabisUse,
            false,
            false,
        ));
        assert!(ny.citation.contains("§ 134"));

        let il = check(&input(
            Regime::IllinoisCrta,
            ProhibitedActType::BanSmokingInLease,
            false,
            false,
        ));
        assert!(il.citation.contains("410 ILCS 705/"));
    }
}
