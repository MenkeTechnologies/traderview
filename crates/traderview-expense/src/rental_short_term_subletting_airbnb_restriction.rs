//! Rental short-term subletting (Airbnb / VRBO / Booking.com) restriction
//! framework — covers (1) tenant subletting on STR platforms WITHOUT
//! landlord written consent (breach of lease assignment / subletting
//! clause), (2) tenant operation of STR violating municipal STR registration
//! requirement, (3) tenant primary-residence requirement (NYC, LA, Boston),
//! (4) lease-clause enforceability under state-mandatory disclosure rules,
//! and (5) landlord enforcement remedies including unlawful detainer.
//!
//! Distinct from sibling `rental_lease_guarantor_disclosure` (lease
//! cosigner separate framework), `tenant_voting_address_protection`
//! (iter 505 ACP confidentiality during STR enforcement), [[tenant_smart_
//! lock_biometric_consent]] (STR-installed smart locks separate framework),
//! `rental_storage_unit_lease_disclosure` (iter 509 storage lease
//! pattern).
//!
//! Trader-landlord critical because (1) **NYC Local Law 18 of 2021**
//! (effective September 5, 2023) requires Mayor's Office of Special
//! Enforcement (OSE) registration plus $145 non-refundable registration fee
//! plus PRIMARY-RESIDENCE host present plus < 30 days rental — Class A
//! multifamily entire-unit STR effectively banned; STR listings in NYC
//! plunged > 90% post-effective date per Brick Underground tracking;
//! (2) **Los Angeles Home-Sharing Ordinance** requires primary-residence
//! host only + 120-day annual cap + City Planning registration + permit
//! number displayed on listing + landlord written approval for renters;
//! (3) **San Francisco Office of Short-Term Rentals** registration fee
//! $925 (2025) + dual registration with Treasurer-Tax Collector + business
//! license + 90-night annual cap for non-hosted; (4) **Boston Short-Term
//! Rental Ordinance** (Mass. Gen. Laws ch. 64G plus Boston Ord. § 9-14)
//! three categories: limited share, home share, owner-adjacent — all
//! require permanent residency; (5) STR tax registration triggers MA
//! Room Occupancy Excise Tax (5.7% state + locally-set additional 6%
//! Boston) + Cape Cod Wastewater fee (2.75%) + Cape Cod & Islands
//! Community Impact (3%).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    NewYorkCity,
    LosAngeles,
    SanFrancisco,
    Boston,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeaseTerm {
    /// Lease has explicit no-subletting / no-Airbnb clause.
    ExplicitNoSublettingAirbnbClause,
    /// Lease requires landlord written consent for subletting.
    SublettingRequiresLandlordConsent,
    /// Lease silent on subletting — fall back to state default rule.
    LeaseSilentDefaultStateRule,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrStatus {
    /// Tenant operating STR without landlord knowledge or consent.
    OperatingWithoutLandlordConsent,
    /// Tenant operating STR with written landlord consent.
    OperatingWithWrittenLandlordConsent,
    /// Tenant not operating STR.
    NotOperatingStr,
    /// Tenant in primary-residence host-present limited share (NYC LL18).
    PrimaryResidenceHostPresentLimitedShare,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    CompliantNoStr,
    CompliantWithLandlordConsentAndRegistration,
    LeaseBreachUnauthorizedSublet,
    MunicipalStrRegistrationMissing,
    PrimaryResidenceRequirementViolated,
    AnnualNightCapExceeded,
    RoomOccupancyExciseTaxNotCollected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub lease_term: LeaseTerm,
    pub str_status: StrStatus,
    pub municipal_registration_obtained: bool,
    pub host_primary_residence_at_least_183_days: bool,
    pub guest_nights_rented_annually: u32,
    pub host_present_during_guest_stay: bool,
    pub room_occupancy_excise_tax_collected_and_remitted: bool,
    pub annual_str_revenue_cents: u64,
    pub annual_rent_cents: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub recommended_actions: Vec<String>,
    pub annual_rent_at_risk_cents: u64,
    pub estimated_str_revenue_disgorgement_cents: u64,
    pub registration_fee_cents: u64,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub const NYC_LL18_EFFECTIVE_DATE: &str = "2023-09-05";
pub const NYC_REGISTRATION_FEE_CENTS: u64 = 14_500;
pub const SF_REGISTRATION_FEE_CENTS: u64 = 92_500;
pub const LA_ANNUAL_NIGHT_CAP: u32 = 120;
pub const SF_NON_HOSTED_ANNUAL_NIGHT_CAP: u32 = 90;
pub const PRIMARY_RESIDENCE_DAYS_PER_YEAR: u32 = 183;
pub const NYC_LL18_MAX_GUEST_NIGHTS_PER_RENTAL_DAYS: u32 = 30;
pub const MA_ROOM_OCCUPANCY_EXCISE_RATE_BPS: u32 = 570;

pub fn check(input: &Input) -> Output {
    let mut actions: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    let severity: Severity;

    if matches!(input.str_status, StrStatus::NotOperatingStr) {
        notes.push(
            "Tenant not operating STR — framework inapplicable. Standard lease terms govern; \
             confirm lease language remains compliant with state subletting-consent rules."
                .to_string(),
        );
        return Output {
            severity: Severity::CompliantNoStr,
            recommended_actions: actions,
            annual_rent_at_risk_cents: 0,
            estimated_str_revenue_disgorgement_cents: 0,
            registration_fee_cents: 0,
            citation: match input.jurisdiction {
                Jurisdiction::NewYorkCity => "NYC LL18 of 2021; NY RPL § 226-b",
                Jurisdiction::LosAngeles => "LAMC § 12.22 A.32 Home-Sharing Ordinance",
                Jurisdiction::SanFrancisco => "SF Admin Code Ch. 41A",
                Jurisdiction::Boston => "Boston Ord. § 9-14",
                Jurisdiction::Default => "Common-law lease assignment rules",
            },
            notes,
        };
    }

    let lease_authorizes = matches!(
        input.str_status,
        StrStatus::OperatingWithWrittenLandlordConsent
            | StrStatus::PrimaryResidenceHostPresentLimitedShare
    ) || matches!(input.lease_term, LeaseTerm::LeaseSilentDefaultStateRule);

    if matches!(input.str_status, StrStatus::OperatingWithoutLandlordConsent)
        && matches!(
            input.lease_term,
            LeaseTerm::ExplicitNoSublettingAirbnbClause
                | LeaseTerm::SublettingRequiresLandlordConsent
        )
    {
        severity = Severity::LeaseBreachUnauthorizedSublet;
        actions.push(
            "Tenant operating STR WITHOUT landlord written consent breaches lease no-\
             subletting / consent-required clause. Landlord remedies: (1) serve 3-day Notice \
             to Cure (CA Code Civ. Proc. § 1161(3)) or equivalent state-law cure notice, \
             (2) if uncured, file unlawful detainer plus seek possession plus damages, (3) \
             potentially seek STR revenue disgorgement as restitution under common-law \
             unjust enrichment. Document STR listing screenshots plus guest reviews plus \
             revenue estimates."
                .to_string(),
        );
    } else if matches!(input.jurisdiction, Jurisdiction::NewYorkCity)
        && lease_authorizes
        && !input.municipal_registration_obtained
    {
        severity = Severity::MunicipalStrRegistrationMissing;
        actions.push(format!(
            "NYC LL18 of 2021 (effective {}) requires Mayor's Office of Special Enforcement \
             (OSE) registration prior to listing on Airbnb / VRBO / Booking.com; ${} \
             non-refundable registration fee. Booking platforms PROHIBITED from processing \
             transactions for unregistered units per LL18 enforcement framework. STR listings \
             in NYC plunged > 90% post-effective date.",
            NYC_LL18_EFFECTIVE_DATE,
            NYC_REGISTRATION_FEE_CENTS / 100
        ));
    } else if (matches!(input.jurisdiction, Jurisdiction::NewYorkCity)
        || matches!(input.jurisdiction, Jurisdiction::LosAngeles)
        || matches!(input.jurisdiction, Jurisdiction::Boston))
        && lease_authorizes
        && !input.host_primary_residence_at_least_183_days
    {
        severity = Severity::PrimaryResidenceRequirementViolated;
        actions.push(format!(
            "Primary-residence requirement violated: host must reside in unit at least {} \
             days per year. NYC LL18 + LA Home-Sharing Ordinance + Boston STR Ordinance \
             all require permanent-residency; Class A multifamily entire-unit STR without \
             host presence is prohibited. Cease STR operation OR document compliant primary \
             residency with utility bills + drivers license + voter registration evidence.",
            PRIMARY_RESIDENCE_DAYS_PER_YEAR
        ));
    } else if matches!(input.jurisdiction, Jurisdiction::LosAngeles)
        && lease_authorizes
        && input.guest_nights_rented_annually > LA_ANNUAL_NIGHT_CAP
    {
        severity = Severity::AnnualNightCapExceeded;
        actions.push(format!(
            "Los Angeles Home-Sharing Ordinance annual cap of {} nights exceeded ({} \
             nights rented). Suspend STR operation; pay applicable per-night fines per \
             LAMC § 12.22 A.32. Extended Home-Sharing permit available for hosts who \
             demonstrate compliance with primary-residence requirement and prior 12-month \
             compliance history.",
            LA_ANNUAL_NIGHT_CAP, input.guest_nights_rented_annually
        ));
    } else if matches!(input.jurisdiction, Jurisdiction::SanFrancisco)
        && lease_authorizes
        && !input.host_present_during_guest_stay
        && input.guest_nights_rented_annually > SF_NON_HOSTED_ANNUAL_NIGHT_CAP
    {
        severity = Severity::AnnualNightCapExceeded;
        actions.push(format!(
            "San Francisco non-hosted (host-absent) annual cap of {} nights exceeded ({} \
             nights). Hosted (host-present) rentals not capped. Cease non-hosted operation \
             or transition to hosted model.",
            SF_NON_HOSTED_ANNUAL_NIGHT_CAP, input.guest_nights_rented_annually
        ));
    } else if matches!(input.jurisdiction, Jurisdiction::Boston)
        && lease_authorizes
        && !input.room_occupancy_excise_tax_collected_and_remitted
    {
        severity = Severity::RoomOccupancyExciseTaxNotCollected;
        actions.push(
            "MA Room Occupancy Excise Tax (5.7% state + 6% Boston locally-set additional = \
             11.7% effective rate per Mass. Gen. Laws ch. 64G) not collected and remitted. \
             Register with MA Department of Revenue MassTaxConnect; collect tax from STR \
             guests; remit monthly via Form ST-7 (Room Occupancy Excise Return); estimated \
             tax liability = annual STR revenue × 5.7% state rate. Cape Cod and Islands \
             additional fees apply if applicable: 2.75% Wastewater fee + 3% Community Impact \
             fee."
                .to_string(),
        );
    } else {
        severity = Severity::CompliantWithLandlordConsentAndRegistration;
        actions.push(
            "Compliant: tenant operating STR with written landlord consent, municipal \
             registration current, primary-residence requirement met, annual night cap not \
             exceeded, applicable excise taxes collected and remitted. Maintain documentation \
             for full statute-of-limitations window (typically 3-6 years state-specific) plus \
             retain platform-host transaction records."
                .to_string(),
        );
    }

    match input.jurisdiction {
        Jurisdiction::NewYorkCity => {
            notes.push(format!(
                "NYC Local Law 18 of 2021 (effective {}) administered by Mayor's Office of \
                 Special Enforcement (OSE); ${} non-refundable registration fee per 6 RCNY \
                 § 1-04. NY RPL § 226-b governs tenant subletting in rent-stabilized units; \
                 NYC Admin Code § 27-265 multiple dwelling Class A vs Class B classification. \
                 Booking platforms (Airbnb, VRBO, Booking.com) PROHIBITED from processing \
                 transactions for unregistered units; listings plunged greater than 90% \
                 post-effective date per Brick Underground tracking.",
                NYC_LL18_EFFECTIVE_DATE,
                NYC_REGISTRATION_FEE_CENTS / 100
            ));
        }
        Jurisdiction::LosAngeles => {
            notes.push(format!(
                "LAMC § 12.22 A.32 Home-Sharing Ordinance enforced by Department of City \
                 Planning. Primary-residence requirement plus {}-night annual cap. Extended \
                 Home-Sharing permit available for compliant hosts. Landlord written \
                 approval required for renter-hosts. LA County Treasurer-Tax Collector \
                 enforces Transient Occupancy Tax (TOT) per LA County Code § 4.72.",
                LA_ANNUAL_NIGHT_CAP
            ));
        }
        Jurisdiction::SanFrancisco => {
            notes.push(format!(
                "SF Office of Short-Term Rentals administered under SF Admin Code Ch. 41A; \
                 ${} registration fee (2025); dual registration with Treasurer-Tax Collector \
                 plus business license. Non-hosted rentals capped at {} nights annually; \
                 hosted rentals uncapped. SF Rent Ordinance § 37 governs subletting; \
                 landlord may evict for unauthorized subletting per Costa-Hawkins Rental \
                 Housing Act.",
                SF_REGISTRATION_FEE_CENTS / 100,
                SF_NON_HOSTED_ANNUAL_NIGHT_CAP
            ));
        }
        Jurisdiction::Boston => {
            notes.push(format!(
                "Boston Ord. § 9-14 STR ordinance; Mass. Gen. Laws ch. 64G Room Occupancy \
                 Excise Tax ({}% state rate); Boston locally-set additional rate 6% per Mass. \
                 Gen. Laws ch. 64G § 3A. Three categories: limited share, home share, owner-\
                 adjacent — all require permanent residency. Cape Cod and Islands Water \
                 Protection Fund additional 2.75% Wastewater fee; Community Impact fee 3%.",
                MA_ROOM_OCCUPANCY_EXCISE_RATE_BPS / 100
            ));
        }
        Jurisdiction::Default => {
            notes.push(
                "Outside NYC / LA / SF / Boston, STR regulation varies widely by state and \
                 municipality. Most jurisdictions require business-license registration plus \
                 transient occupancy tax (TOT) / hotel occupancy tax collection; many require \
                 STR-specific permit. Common-law lease assignment rules permit landlord \
                 enforcement of no-subletting clause via unlawful detainer."
                    .to_string(),
            );
        }
    }

    notes.push(
        "Coordination with [[rental_lease_guarantor_disclosure]] (lease cosigner — \
         distinct subletting analysis), [[tenant_voting_address_protection]] (iter 505 — \
         ACP participant confidentiality during STR enforcement action), [[tenant_smart_\
         lock_biometric_consent]] (STR-installed smart locks separate biometric framework), \
         [[rental_storage_unit_lease_disclosure]] (iter 509 — STR units may have separate \
         storage component), [[tenant_emotional_distress_damages]] (IIED claim for \
         malicious STR-enforcement action), [[mid_tenancy_temporary_relocation]] (when \
         primary-resident host displaces self during STR rental window)."
            .to_string(),
    );

    let str_revenue_disgorgement: u64 = match severity {
        Severity::LeaseBreachUnauthorizedSublet => input.annual_str_revenue_cents,
        _ => 0,
    };

    let registration_fee = match input.jurisdiction {
        Jurisdiction::NewYorkCity => NYC_REGISTRATION_FEE_CENTS,
        Jurisdiction::SanFrancisco => SF_REGISTRATION_FEE_CENTS,
        _ => 0,
    };

    let annual_rent_at_risk: u64 = match severity {
        Severity::LeaseBreachUnauthorizedSublet => input.annual_rent_cents,
        Severity::PrimaryResidenceRequirementViolated
        | Severity::MunicipalStrRegistrationMissing
        | Severity::AnnualNightCapExceeded
        | Severity::RoomOccupancyExciseTaxNotCollected => input.annual_rent_cents.saturating_div(2),
        _ => 0,
    };

    Output {
        severity,
        recommended_actions: actions,
        annual_rent_at_risk_cents: annual_rent_at_risk,
        estimated_str_revenue_disgorgement_cents: str_revenue_disgorgement,
        registration_fee_cents: registration_fee,
        citation: match input.jurisdiction {
            Jurisdiction::NewYorkCity => "NYC LL18 of 2021 + NY RPL § 226-b + 6 RCNY § 1-04",
            Jurisdiction::LosAngeles => "LAMC § 12.22 A.32 + LA County Code § 4.72 TOT",
            Jurisdiction::SanFrancisco => "SF Admin Code Ch. 41A + SF Rent Ordinance § 37",
            Jurisdiction::Boston => "Boston Ord. § 9-14 + Mass. Gen. Laws ch. 64G + § 3A",
            Jurisdiction::Default => "Common-law lease assignment + state TOT framework",
        },
        notes,
    }
}

pub type RentalShortTermSublettingAirbnbRestrictionInput = Input;
pub type RentalShortTermSublettingAirbnbRestrictionResult = Output;

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Input {
        Input {
            jurisdiction: Jurisdiction::NewYorkCity,
            lease_term: LeaseTerm::SublettingRequiresLandlordConsent,
            str_status: StrStatus::OperatingWithWrittenLandlordConsent,
            municipal_registration_obtained: true,
            host_primary_residence_at_least_183_days: true,
            guest_nights_rented_annually: 30,
            host_present_during_guest_stay: true,
            room_occupancy_excise_tax_collected_and_remitted: true,
            annual_str_revenue_cents: 50_000_00,
            annual_rent_cents: 60_000_00,
        }
    }

    #[test]
    fn tenant_not_operating_str_compliant() {
        let mut i = baseline();
        i.str_status = StrStatus::NotOperatingStr;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::CompliantNoStr));
        assert_eq!(r.annual_rent_at_risk_cents, 0);
    }

    #[test]
    fn compliant_with_consent_and_registration() {
        let i = baseline();
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::CompliantWithLandlordConsentAndRegistration
        ));
        assert_eq!(r.annual_rent_at_risk_cents, 0);
    }

    #[test]
    fn unauthorized_sublet_lease_breach_full_rent_at_risk() {
        let mut i = baseline();
        i.str_status = StrStatus::OperatingWithoutLandlordConsent;
        i.lease_term = LeaseTerm::ExplicitNoSublettingAirbnbClause;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::LeaseBreachUnauthorizedSublet
        ));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents);
        assert_eq!(
            r.estimated_str_revenue_disgorgement_cents,
            i.annual_str_revenue_cents
        );
    }

    #[test]
    fn nyc_no_registration_violation() {
        let mut i = baseline();
        i.municipal_registration_obtained = false;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::MunicipalStrRegistrationMissing
        ));
        assert!(r.recommended_actions.iter().any(|a| a.contains("LL18")));
        assert_eq!(r.registration_fee_cents, NYC_REGISTRATION_FEE_CENTS);
    }

    #[test]
    fn primary_residence_requirement_violated_nyc() {
        let mut i = baseline();
        i.host_primary_residence_at_least_183_days = false;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::PrimaryResidenceRequirementViolated
        ));
        assert!(r.recommended_actions.iter().any(|a| a.contains("183")));
    }

    #[test]
    fn la_annual_night_cap_exceeded_violation() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::LosAngeles;
        i.guest_nights_rented_annually = 150;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::AnnualNightCapExceeded));
        assert!(r.recommended_actions.iter().any(|a| a.contains("120")));
    }

    #[test]
    fn la_annual_night_cap_at_120_compliant() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::LosAngeles;
        i.guest_nights_rented_annually = 120;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::CompliantWithLandlordConsentAndRegistration
        ));
    }

    #[test]
    fn sf_non_hosted_cap_exceeded_violation() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::SanFrancisco;
        i.host_present_during_guest_stay = false;
        i.guest_nights_rented_annually = 100;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::AnnualNightCapExceeded));
        assert!(r.recommended_actions.iter().any(|a| a.contains("90")));
    }

    #[test]
    fn sf_hosted_uncapped_compliant() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::SanFrancisco;
        i.host_present_during_guest_stay = true;
        i.guest_nights_rented_annually = 200;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::CompliantWithLandlordConsentAndRegistration
        ));
    }

    #[test]
    fn boston_excise_tax_not_remitted_violation() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Boston;
        i.room_occupancy_excise_tax_collected_and_remitted = false;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::RoomOccupancyExciseTaxNotCollected
        ));
        assert!(r.recommended_actions.iter().any(|a| a.contains("5.7%")));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("Mass. Gen. Laws ch. 64G")));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("Form ST-7")));
    }

    #[test]
    fn nyc_jurisdiction_pins_ll18_effective_2023_09_05() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("Local Law 18 of 2021")));
        assert!(r.notes.iter().any(|n| n.contains("2023-09-05")));
        assert!(r.notes.iter().any(|n| n.contains("Brick Underground")));
    }

    #[test]
    fn la_jurisdiction_pins_home_sharing_ordinance_and_120_cap() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::LosAngeles;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("LAMC § 12.22 A.32")));
        assert!(r.notes.iter().any(|n| n.contains("120")));
        assert!(r.notes.iter().any(|n| n.contains("LA County Code § 4.72")));
    }

    #[test]
    fn sf_jurisdiction_pins_ch_41a_and_925_dollar_fee() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::SanFrancisco;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("Ch. 41A")));
        assert!(r.notes.iter().any(|n| n.contains("925")));
        assert!(r.notes.iter().any(|n| n.contains("Costa-Hawkins")));
    }

    #[test]
    fn boston_jurisdiction_pins_64g_and_cape_cod_fees() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Boston;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("Boston Ord. § 9-14")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Mass. Gen. Laws ch. 64G")));
        assert!(r.notes.iter().any(|n| n.contains("Cape Cod")));
        assert!(r.notes.iter().any(|n| n.contains("2.75%")));
    }

    #[test]
    fn default_jurisdiction_pins_common_law_tot() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Default;
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("transient occupancy tax")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Common-law lease assignment")));
    }

    #[test]
    fn coordination_note_references_siblings() {
        let i = baseline();
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("rental_lease_guarantor_disclosure")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("tenant_voting_address_protection")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("tenant_smart_lock_biometric_consent")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("mid_tenancy_temporary_relocation")));
    }

    #[test]
    fn coordination_note_pinned_in_all_jurisdictions() {
        for j in [
            Jurisdiction::NewYorkCity,
            Jurisdiction::LosAngeles,
            Jurisdiction::SanFrancisco,
            Jurisdiction::Boston,
            Jurisdiction::Default,
        ] {
            let mut i = baseline();
            i.jurisdiction = j;
            let r = check(&i);
            assert!(
                r.notes
                    .iter()
                    .any(|n| n.contains("rental_lease_guarantor_disclosure")),
                "coordination missing for {j:?}"
            );
        }
    }

    #[test]
    fn nyc_ll18_effective_date_pins_2023_09_05() {
        assert_eq!(NYC_LL18_EFFECTIVE_DATE, "2023-09-05");
    }

    #[test]
    fn nyc_registration_fee_pins_145_dollars() {
        assert_eq!(NYC_REGISTRATION_FEE_CENTS, 14_500);
    }

    #[test]
    fn sf_registration_fee_pins_925_dollars() {
        assert_eq!(SF_REGISTRATION_FEE_CENTS, 92_500);
    }

    #[test]
    fn la_annual_night_cap_pins_120() {
        assert_eq!(LA_ANNUAL_NIGHT_CAP, 120);
    }

    #[test]
    fn sf_non_hosted_cap_pins_90() {
        assert_eq!(SF_NON_HOSTED_ANNUAL_NIGHT_CAP, 90);
    }

    #[test]
    fn primary_residence_days_pins_183() {
        assert_eq!(PRIMARY_RESIDENCE_DAYS_PER_YEAR, 183);
    }

    #[test]
    fn ma_room_occupancy_rate_pins_570_bps() {
        assert_eq!(MA_ROOM_OCCUPANCY_EXCISE_RATE_BPS, 570);
    }

    #[test]
    fn citation_branch_for_each_jurisdiction() {
        let nyc = check(&{
            let mut i = baseline();
            i.jurisdiction = Jurisdiction::NewYorkCity;
            i
        });
        let la = check(&{
            let mut i = baseline();
            i.jurisdiction = Jurisdiction::LosAngeles;
            i
        });
        let sf = check(&{
            let mut i = baseline();
            i.jurisdiction = Jurisdiction::SanFrancisco;
            i
        });
        let bos = check(&{
            let mut i = baseline();
            i.jurisdiction = Jurisdiction::Boston;
            i
        });
        let de = check(&{
            let mut i = baseline();
            i.jurisdiction = Jurisdiction::Default;
            i
        });
        assert!(nyc.citation.contains("LL18"));
        assert!(la.citation.contains("LAMC"));
        assert!(sf.citation.contains("Ch. 41A"));
        assert!(bos.citation.contains("ch. 64G"));
        assert!(de.citation.contains("Common-law"));
    }

    #[test]
    fn severity_priority_unauthorized_sublet_overrides_registration() {
        let mut i = baseline();
        i.str_status = StrStatus::OperatingWithoutLandlordConsent;
        i.lease_term = LeaseTerm::ExplicitNoSublettingAirbnbClause;
        i.municipal_registration_obtained = false;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::LeaseBreachUnauthorizedSublet
        ));
    }

    #[test]
    fn zero_rent_zero_risk_does_not_panic() {
        let mut i = baseline();
        i.annual_rent_cents = 0;
        i.str_status = StrStatus::OperatingWithoutLandlordConsent;
        i.lease_term = LeaseTerm::ExplicitNoSublettingAirbnbClause;
        let r = check(&i);
        assert_eq!(r.annual_rent_at_risk_cents, 0);
    }

    #[test]
    fn limited_share_primary_residence_host_present_compliant() {
        let mut i = baseline();
        i.str_status = StrStatus::PrimaryResidenceHostPresentLimitedShare;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::CompliantWithLandlordConsentAndRegistration
        ));
    }
}
