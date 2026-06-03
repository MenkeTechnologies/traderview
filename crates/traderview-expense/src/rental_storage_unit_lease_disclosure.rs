//! Rental storage unit lease disclosure framework — covers (1) commercial
//! self-service storage facilities (subject to state Self-Service Storage
//! Facility Acts plus newly-enacted California SB 709 first-page disclosure
//! requirements effective January 1, 2026) plus (2) residential rental
//! storage (basement, garage, external locker leased to apartment tenant
//! alongside dwelling — governed by landlord-tenant law plus state-specific
//! lien limits on tenant property).
//!
//! Distinct from sibling `rental_basement_water_intrusion_disclosure`
//! (basement water disclosure for storage area), [[rental_chimney_fireplace_
//! inspection_disclosure]] (chimney near attic storage), [[rental_propane_
//! tank_lease_disclosure]] (lease-pass-through analog), [[rental_in_unit_
//! laundry_appliance_provision]] (iter 501 in-unit appliance framework),
//! `tenant_voting_address_protection` (iter 505 confidentiality).
//!
//! Trader-landlord critical because (1) **California SB 709 effective
//! January 1, 2026** amends Cal. Bus. & Prof. Code § 21712 to require
//! six prominent first-page disclosures in commercial self-storage rental
//! agreements: (A) length of rental agreement and renewal terms, (B)
//! whether rental fee is promotional or discounted, (C) duration of any
//! promotional rate, (D) whether rental fee can change and maximum fee
//! during first 12 months, (E) specific steps to terminate and avoid
//! future fees, (F) owner's contact information; (2) Cal. Bus. & Prof.
//! Code § 21702(c) self-service storage facility definition EXPLICITLY
//! EXCLUDES "garages and other storage areas in private residences" —
//! meaning rental-storage-with-apartment is NOT covered by SSF Act and
//! is governed instead by landlord-tenant habitability law plus UCC
//! Article 7 warehouse-keeper lien (if applicable); (3) New York Senate
//! Bill S3690 (2025 session, pending) would expand pre-lien-enforcement
//! notice requirements; (4) Florida Fla. Stat. § 83.801 et seq. + Texas
//! Tex. Prop. Code § 59.001 govern Self-Service Storage in those states.
//!
//! Lien framework: California 14-day default trigger, 60-day notice to
//! occupant via certified mail before sale, second-class mail notice plus
//! newspaper publication, sale by competitive bid (or licensed auctioneer),
//! identity documents must be returned to occupant for at least 60 days
//! post-sale. Excess proceeds after lien satisfaction returned to occupant
//! within 90 days; unclaimed proceeds escheat to state unclaimed-property
//! fund.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    California,
    NewYork,
    Florida,
    Texas,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageType {
    /// Commercial self-service storage facility — subject to SSF Act +
    /// SB 709 (CA).
    CommercialSelfServiceStorageFacility,
    /// Residential storage included with apartment lease (basement,
    /// garage, exterior locker) — landlord-tenant law + UCC Article 7.
    ResidentialStorageIncludedWithApartmentLease,
    /// Separate residential storage rental on same premises but distinct
    /// from apartment lease.
    SeparateResidentialStorageRental,
    /// No storage unit rented.
    NoStorageUnit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LienAction {
    /// No lien action being taken.
    NoLienAction,
    /// Default has occurred but cure window has not lapsed.
    DefaultWithinCureWindow,
    /// Lien-sale notice has been issued.
    LienSaleNoticeIssued,
    /// Lien sale completed.
    LienSaleCompleted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    CompliantFullDisclosure,
    SbSix709FirstPageDisclosureMissing,
    LienDefaultCureWindowNotMet,
    LienSaleNoticeRequirementsNotMet,
    IdentityDocumentReturnRequirementBreached,
    ExcessProceedsReturnFailureToTenant,
    SsfActInapplicabilityResidentialStorage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub storage_type: StorageType,
    pub lease_year: i32,
    pub lease_signed_on_or_after_2026_01_01: bool,
    pub sb_709_six_disclosures_on_first_page: bool,
    pub lien_action: LienAction,
    pub days_since_default: u32,
    pub days_since_lien_sale_notice: u32,
    pub identity_documents_returned_within_60_days_post_sale: bool,
    pub excess_proceeds_returned_to_tenant_within_90_days: bool,
    pub monthly_rental_cents: u64,
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

pub const CA_SB_709_EFFECTIVE_YEAR: i32 = 2026;
pub const CA_LIEN_DEFAULT_CURE_DAYS: u32 = 14;
pub const CA_LIEN_SALE_NOTICE_DAYS: u32 = 60;
pub const IDENTITY_DOC_RETURN_WINDOW_DAYS: u32 = 60;
pub const EXCESS_PROCEEDS_RETURN_WINDOW_DAYS: u32 = 90;

pub fn check(input: &Input) -> Output {
    let mut actions: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    let severity: Severity;

    if matches!(input.storage_type, StorageType::NoStorageUnit) {
        notes.push(
            "No storage unit rented — framework inapplicable. Routine landlord-tenant lease \
             confidentiality and habitability apply to dwelling unit only."
                .to_string(),
        );
        return Output {
            severity: Severity::NotApplicable,
            recommended_actions: actions,
            annual_rent_at_risk_cents: 0,
            citation: "n/a",
            notes,
        };
    }

    if matches!(
        input.storage_type,
        StorageType::ResidentialStorageIncludedWithApartmentLease
            | StorageType::SeparateResidentialStorageRental
    ) {
        severity = Severity::SsfActInapplicabilityResidentialStorage;
        actions.push(
            "Residential storage rental (basement, garage, exterior locker) provided to \
             apartment tenant is EXPLICITLY EXCLUDED from Self-Service Storage Facility Act \
             coverage per Cal. Bus. & Prof. Code § 21702(c). Governed instead by landlord-\
             tenant habitability framework plus UCC Article 7 warehouse-keeper lien if \
             applicable. Cannot invoke SSF Act lien procedures; must use ordinary unlawful-\
             detainer action plus separate landlord-lien claim under Cal. Civ. Code § \
             1980-1991 (CA — abandoned personal property) or state-law equivalent for \
             stored tenant property."
                .to_string(),
        );
        notes.push(
            "Critical compliance distinction: residential rental storage attached to \
             apartment lease must follow ordinary security-deposit rules under state law, \
             not the more aggressive lien-and-sale procedures available to commercial self-\
             storage facilities. Misuse of SSF lien procedures against residential tenant \
             stored property exposes landlord to conversion liability plus statutory \
             damages."
                .to_string(),
        );
    } else if matches!(input.jurisdiction, Jurisdiction::California)
        && input.lease_signed_on_or_after_2026_01_01
        && !input.sb_709_six_disclosures_on_first_page
    {
        severity = Severity::SbSix709FirstPageDisclosureMissing;
        actions.push(format!(
            "SB 709 effective January 1, {} requires six prominent first-page disclosures in \
             commercial self-storage rental agreements per Cal. Bus. & Prof. Code § 21712: \
             (A) length of rental agreement plus renewal terms, (B) whether rental fee is \
             promotional or discounted, (C) duration of any promotional rate, (D) whether \
             rental fee can change and maximum fee during first 12 months, (E) specific \
             steps to terminate and avoid future fees, (F) owner's contact information. \
             Revise rental agreement template plus issue addendum to existing tenants for \
             renewals occurring on or after effective date.",
            CA_SB_709_EFFECTIVE_YEAR
        ));
    } else if matches!(input.lien_action, LienAction::DefaultWithinCureWindow)
        && input.days_since_default < CA_LIEN_DEFAULT_CURE_DAYS
    {
        severity = Severity::LienDefaultCureWindowNotMet;
        actions.push(format!(
            "Default cure window not met: {} days since default; statutory minimum is {} \
             consecutive days per Cal. Bus. & Prof. Code § 21705 before lien attaches. \
             DELAY lien-sale notice issuance until cure window expires; meanwhile send \
             courtesy past-due notice via first-class mail to allow occupant opportunity \
             to cure.",
            input.days_since_default, CA_LIEN_DEFAULT_CURE_DAYS
        ));
    } else if matches!(
        input.lien_action,
        LienAction::LienSaleNoticeIssued | LienAction::LienSaleCompleted
    ) && input.days_since_lien_sale_notice < CA_LIEN_SALE_NOTICE_DAYS
    {
        severity = Severity::LienSaleNoticeRequirementsNotMet;
        actions.push(format!(
            "Lien-sale notice timing requirement not met: {} days since notice issued; \
             statutory minimum {} days required between notice and sale per Cal. Bus. & \
             Prof. Code § 21707. Notice must be sent via verified mail (certified mail OR \
             Cal. Bus. & Prof. Code § 21703.1 alternative electronic-delivery rule eff. \
             2018) AND published in newspaper of general circulation OR posted on website \
             qualifying under § 21707(g). POSTPONE sale until full notice window elapses; \
             sale conducted prematurely is VOID and exposes facility to conversion damages.",
            input.days_since_lien_sale_notice, CA_LIEN_SALE_NOTICE_DAYS
        ));
    } else if matches!(input.lien_action, LienAction::LienSaleCompleted)
        && !input.identity_documents_returned_within_60_days_post_sale
    {
        severity = Severity::IdentityDocumentReturnRequirementBreached;
        actions.push(format!(
            "Personal identity documents (driver's license, passport, birth certificate, \
             Social Security card, immigration documents) MUST be returned to occupant for \
             at least {} days post-sale per Cal. Bus. & Prof. Code § 21709 + comparable \
             state statutes. Sale of identity documents creates criminal exposure under \
             California Penal Code § 530.5 identity theft framework and federal 18 U.S.C. § \
             1028 identification document fraud. IMMEDIATE retrieval from purchaser plus \
             return to occupant required.",
            IDENTITY_DOC_RETURN_WINDOW_DAYS
        ));
    } else if matches!(input.lien_action, LienAction::LienSaleCompleted)
        && !input.excess_proceeds_returned_to_tenant_within_90_days
    {
        severity = Severity::ExcessProceedsReturnFailureToTenant;
        actions.push(format!(
            "Excess sale proceeds after lien satisfaction NOT returned to occupant within \
             {} days per Cal. Bus. & Prof. Code § 21710. Calculate net proceeds (sale price \
             minus past-due rent minus late fees minus advertising costs minus sale costs), \
             remit balance to occupant via certified mail to last-known address. Unclaimed \
             proceeds after additional statutory period escheat to state unclaimed-property \
             fund under Cal. Civ. Code § 1500 et seq. Failure to remit exposes facility to \
             conversion damages.",
            EXCESS_PROCEEDS_RETURN_WINDOW_DAYS
        ));
    } else {
        severity = Severity::CompliantFullDisclosure;
        actions.push(format!(
            "Compliant: SB 709 six first-page disclosures provided (CA leases on or after \
             January 1, {}); lien procedures follow Cal. Bus. & Prof. Code § 21700-21716 \
             timing (default cure {} days, notice {} days, identity doc return {} days, \
             excess proceeds return {} days). Retain documentation in facility file for {} \
             years post-sale per § 21712.5 recordkeeping requirement.",
            CA_SB_709_EFFECTIVE_YEAR,
            CA_LIEN_DEFAULT_CURE_DAYS,
            CA_LIEN_SALE_NOTICE_DAYS,
            IDENTITY_DOC_RETURN_WINDOW_DAYS,
            EXCESS_PROCEEDS_RETURN_WINDOW_DAYS,
            7
        ));
    }

    match input.jurisdiction {
        Jurisdiction::California => {
            notes.push(format!(
                "Cal. Bus. & Prof. Code § 21700-21716 Self-Service Storage Facility Act; \
                 SB 709 amends § 21712 effective January 1, {} adding six first-page \
                 disclosure requirements. AB 1916 (2024) and AB 1108 (2018) prior \
                 amendments. § 21702(c) EXPLICITLY EXCLUDES garages and other storage \
                 areas in private residences from SSF Act coverage.",
                CA_SB_709_EFFECTIVE_YEAR
            ));
        }
        Jurisdiction::NewYork => {
            notes.push(
                "N.Y. General Business Law § 182 Self-Service Storage Facility Act; New \
                 York Senate Bill S3690 (2025 session, pending) would expand pre-lien-\
                 enforcement notice requirements. N.Y. Lien Law § 184 contains warehouse-\
                 keeper lien framework applicable to non-self-storage facility scenarios."
                    .to_string(),
            );
        }
        Jurisdiction::Florida => {
            notes.push(
                "Fla. Stat. § 83.801-83.809 Self-Service Storage Space Act; Florida lien \
                 default trigger 7 days (shorter than California 14 days); newspaper \
                 publication required twice in successive weeks at least 7 days apart per \
                 § 83.806."
                    .to_string(),
            );
        }
        Jurisdiction::Texas => {
            notes.push(
                "Tex. Prop. Code § 59.001-59.046 Self-Service Storage Liens; lessor must \
                 send notice via verified mail to last-known address before exercising \
                 lien per § 59.044; identity documents (driver's license, passport) must \
                 be returned to tenant per § 59.041(c)."
                    .to_string(),
            );
        }
        Jurisdiction::Default => {
            notes.push(
                "Uniform Self-Service Storage Facility Act adopted by majority of US \
                 states with state-specific variations; default to UCC Article 7 \
                 warehouse-keeper lien framework where state has not enacted dedicated \
                 SSF statute. UCC § 7-209 warehouse lien for charges plus UCC § 7-210 \
                 enforcement procedure."
                    .to_string(),
            );
        }
    }

    notes.push(
        "Coordination with [[rental_basement_water_intrusion_disclosure]] (basement-\
         storage water-intrusion disclosure cross-reference for stored property damage), \
         [[rental_propane_tank_lease_disclosure]] (lease-pass-through analog), \
         [[rental_in_unit_laundry_appliance_provision]] (laundry-room appliance + \
         storage overlap), [[tenant_voting_address_protection]] (ACP participant address \
         confidentiality during lien-sale newspaper publication — substitute address \
         required for ACP participants), [[mid_tenancy_temporary_relocation]] (when \
         apartment-tenant must relocate stored property during construction), \
         [[tenant_emotional_distress_damages]] (IIED claim for wrongful sale of \
         sentimental items)."
            .to_string(),
    );

    let annual_rent_at_risk: u64 = match severity {
        Severity::IdentityDocumentReturnRequirementBreached
        | Severity::ExcessProceedsReturnFailureToTenant
        | Severity::LienSaleNoticeRequirementsNotMet => input.annual_rent_cents,
        Severity::SbSix709FirstPageDisclosureMissing
        | Severity::LienDefaultCureWindowNotMet
        | Severity::SsfActInapplicabilityResidentialStorage => {
            input.annual_rent_cents.saturating_div(2)
        }
        _ => 0,
    };

    Output {
        severity,
        recommended_actions: actions,
        annual_rent_at_risk_cents: annual_rent_at_risk,
        citation: match input.jurisdiction {
            Jurisdiction::California => "Cal. Bus. & Prof. Code § 21700-21716 + SB 709 (eff. 2026)",
            Jurisdiction::NewYork => "N.Y. General Business Law § 182 + Lien Law § 184",
            Jurisdiction::Florida => "Fla. Stat. § 83.801-83.809",
            Jurisdiction::Texas => "Tex. Prop. Code § 59.001-59.046",
            Jurisdiction::Default => "Uniform Self-Service Storage Facility Act + UCC Art. 7",
        },
        notes,
    }
}

pub type RentalStorageUnitLeaseDisclosureInput = Input;
pub type RentalStorageUnitLeaseDisclosureResult = Output;

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Input {
        Input {
            jurisdiction: Jurisdiction::California,
            storage_type: StorageType::CommercialSelfServiceStorageFacility,
            lease_year: 2026,
            lease_signed_on_or_after_2026_01_01: true,
            sb_709_six_disclosures_on_first_page: true,
            lien_action: LienAction::NoLienAction,
            days_since_default: 0,
            days_since_lien_sale_notice: 0,
            identity_documents_returned_within_60_days_post_sale: true,
            excess_proceeds_returned_to_tenant_within_90_days: true,
            monthly_rental_cents: 200_00,
            annual_rent_cents: 2_400_00,
        }
    }

    #[test]
    fn no_storage_unit_not_applicable() {
        let mut i = baseline();
        i.storage_type = StorageType::NoStorageUnit;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NotApplicable));
        assert_eq!(r.annual_rent_at_risk_cents, 0);
    }

    #[test]
    fn residential_storage_excluded_from_ssf_act() {
        let mut i = baseline();
        i.storage_type = StorageType::ResidentialStorageIncludedWithApartmentLease;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::SsfActInapplicabilityResidentialStorage));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("§ 21702(c)")));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("Cal. Civ. Code § 1980-1991")));
    }

    #[test]
    fn separate_residential_storage_also_excluded() {
        let mut i = baseline();
        i.storage_type = StorageType::SeparateResidentialStorageRental;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::SsfActInapplicabilityResidentialStorage));
    }

    #[test]
    fn compliant_full_disclosure() {
        let i = baseline();
        let r = check(&i);
        assert!(matches!(r.severity, Severity::CompliantFullDisclosure));
        assert_eq!(r.annual_rent_at_risk_cents, 0);
    }

    #[test]
    fn ca_sb_709_post_2026_no_disclosures_violation() {
        let mut i = baseline();
        i.sb_709_six_disclosures_on_first_page = false;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::SbSix709FirstPageDisclosureMissing));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents / 2);
        assert!(r.recommended_actions.iter().any(|a| a.contains("§ 21712")));
    }

    #[test]
    fn ca_pre_2026_no_sb_709_disclosure_compliant() {
        let mut i = baseline();
        i.lease_year = 2025;
        i.lease_signed_on_or_after_2026_01_01 = false;
        i.sb_709_six_disclosures_on_first_page = false;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::CompliantFullDisclosure));
    }

    #[test]
    fn lien_default_within_cure_window_violation() {
        let mut i = baseline();
        i.lien_action = LienAction::DefaultWithinCureWindow;
        i.days_since_default = 7;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::LienDefaultCureWindowNotMet));
        assert!(r.recommended_actions.iter().any(|a| a.contains("§ 21705")));
    }

    #[test]
    fn lien_default_at_14_days_compliant_path() {
        let mut i = baseline();
        i.lien_action = LienAction::DefaultWithinCureWindow;
        i.days_since_default = 14;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::CompliantFullDisclosure));
    }

    #[test]
    fn lien_sale_notice_too_short_violation() {
        let mut i = baseline();
        i.lien_action = LienAction::LienSaleNoticeIssued;
        i.days_since_lien_sale_notice = 30;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::LienSaleNoticeRequirementsNotMet));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents);
        assert!(r.recommended_actions.iter().any(|a| a.contains("§ 21707")));
    }

    #[test]
    fn lien_sale_notice_at_60_days_compliant() {
        let mut i = baseline();
        i.lien_action = LienAction::LienSaleNoticeIssued;
        i.days_since_lien_sale_notice = 60;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::CompliantFullDisclosure));
    }

    #[test]
    fn identity_document_not_returned_violation() {
        let mut i = baseline();
        i.lien_action = LienAction::LienSaleCompleted;
        i.days_since_lien_sale_notice = 60;
        i.identity_documents_returned_within_60_days_post_sale = false;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::IdentityDocumentReturnRequirementBreached
        ));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents);
        assert!(r.recommended_actions.iter().any(|a| a.contains("§ 21709")));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("California Penal Code § 530.5")));
        assert!(r.recommended_actions.iter().any(|a| a.contains("18 U.S.C. § 1028")));
    }

    #[test]
    fn excess_proceeds_not_returned_violation() {
        let mut i = baseline();
        i.lien_action = LienAction::LienSaleCompleted;
        i.days_since_lien_sale_notice = 60;
        i.excess_proceeds_returned_to_tenant_within_90_days = false;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::ExcessProceedsReturnFailureToTenant));
        assert!(r.recommended_actions.iter().any(|a| a.contains("§ 21710")));
        assert!(r.recommended_actions.iter().any(|a| a.contains("Cal. Civ. Code § 1500")));
    }

    #[test]
    fn ca_jurisdiction_pins_sb_709_and_21700_21716() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("SB 709")));
        assert!(r.notes.iter().any(|n| n.contains("§ 21700-21716")));
        assert!(r.notes.iter().any(|n| n.contains("§ 21702(c)")));
        assert!(r.notes.iter().any(|n| n.contains("January 1, 2026")));
    }

    #[test]
    fn ny_jurisdiction_pins_general_business_law_182_and_sb_s3690() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::NewYork;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("N.Y. General Business Law § 182")));
        assert!(r.notes.iter().any(|n| n.contains("S3690")));
        assert!(r.notes.iter().any(|n| n.contains("N.Y. Lien Law § 184")));
    }

    #[test]
    fn fl_jurisdiction_pins_fla_stat_83_801() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Florida;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("Fla. Stat. § 83.801-83.809")));
        assert!(r.notes.iter().any(|n| n.contains("7 days")));
    }

    #[test]
    fn tx_jurisdiction_pins_tex_prop_code_59() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Texas;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("Tex. Prop. Code § 59.001-59.046")));
        assert!(r.notes.iter().any(|n| n.contains("§ 59.044")));
    }

    #[test]
    fn default_jurisdiction_pins_ucc_article_7() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Default;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("UCC § 7-209")));
        assert!(r.notes.iter().any(|n| n.contains("UCC § 7-210")));
        assert!(r.notes.iter().any(|n| n.contains("Uniform Self-Service Storage Facility Act")));
    }

    #[test]
    fn coordination_note_references_siblings() {
        let i = baseline();
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("rental_basement_water_intrusion_disclosure")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("tenant_voting_address_protection")));
        assert!(r.notes.iter().any(|n| n.contains("rental_in_unit_laundry_appliance_provision")));
        assert!(r.notes.iter().any(|n| n.contains("tenant_emotional_distress_damages")));
    }

    #[test]
    fn coordination_note_pinned_in_all_jurisdictions() {
        for j in [
            Jurisdiction::California,
            Jurisdiction::NewYork,
            Jurisdiction::Florida,
            Jurisdiction::Texas,
            Jurisdiction::Default,
        ] {
            let mut i = baseline();
            i.jurisdiction = j;
            let r = check(&i);
            assert!(
                r.notes
                    .iter()
                    .any(|n| n.contains("tenant_voting_address_protection")),
                "coordination missing for {j:?}"
            );
        }
    }

    #[test]
    fn ca_sb_709_effective_year_pins_2026() {
        assert_eq!(CA_SB_709_EFFECTIVE_YEAR, 2026);
    }

    #[test]
    fn ca_lien_default_cure_pins_14_days() {
        assert_eq!(CA_LIEN_DEFAULT_CURE_DAYS, 14);
    }

    #[test]
    fn ca_lien_sale_notice_pins_60_days() {
        assert_eq!(CA_LIEN_SALE_NOTICE_DAYS, 60);
    }

    #[test]
    fn identity_doc_return_window_pins_60_days() {
        assert_eq!(IDENTITY_DOC_RETURN_WINDOW_DAYS, 60);
    }

    #[test]
    fn excess_proceeds_return_window_pins_90_days() {
        assert_eq!(EXCESS_PROCEEDS_RETURN_WINDOW_DAYS, 90);
    }

    #[test]
    fn citation_branch_for_each_jurisdiction() {
        let ca = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::California; i });
        let ny = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::NewYork; i });
        let fl = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::Florida; i });
        let tx = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::Texas; i });
        let de = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::Default; i });
        assert!(ca.citation.contains("SB 709"));
        assert!(ny.citation.contains("§ 182"));
        assert!(fl.citation.contains("§ 83.801-83.809"));
        assert!(tx.citation.contains("§ 59.001-59.046"));
        assert!(de.citation.contains("UCC Art. 7"));
    }

    #[test]
    fn severity_priority_residential_overrides_sb_709() {
        let mut i = baseline();
        i.storage_type = StorageType::ResidentialStorageIncludedWithApartmentLease;
        i.sb_709_six_disclosures_on_first_page = false;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::SsfActInapplicabilityResidentialStorage));
    }

    #[test]
    fn zero_rent_zero_risk_does_not_panic() {
        let mut i = baseline();
        i.annual_rent_cents = 0;
        i.sb_709_six_disclosures_on_first_page = false;
        let r = check(&i);
        assert_eq!(r.annual_rent_at_risk_cents, 0);
    }
}
