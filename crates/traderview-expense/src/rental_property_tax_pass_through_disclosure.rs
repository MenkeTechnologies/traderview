//! Rental property tax pass-through disclosure framework — covers when a
//! trader-landlord may pass through property-tax increases (reassessment +
//! special assessments + Major Capital Improvement (MCI) recoveries +
//! local-improvement-district fees) to tenants under state and local rent-
//! control / rent-stabilization regimes.
//!
//! Distinct from sibling [[rental_property_registration]] (registration
//! framework — distinct compliance pathway), [[rental_hoa_disclosure_at_
//! lease]] (HOA fees framework), [[rental_solar_panel_disclosure]] (iter
//! 495 PPA pass-through analog), [[rental_in_unit_laundry_appliance_
//! provision]] (iter 501 appliance maintenance — distinct), [[tenant_late_
//! fee_cap]] (late-fee framework — distinct charge), [[rental_junk_fee_
//! transparency]] (junk-fee disclosure framework — distinct).
//!
//! Trader-landlord critical because (1) **California Proposition 13** (1978)
//! caps property tax increases at 2% per year UNTIL change-of-ownership or
//! new construction triggers full reassessment to market value — non-
//! commercial residential leases generally do NOT pass through Prop-13
//! reassessment to tenants absent express lease clause; (2) **California
//! Costa-Hawkins Rental Housing Act** (Cal. Civ. Code § 1954.50-.535)
//! exempts post-1995 construction + single-family / condominium units from
//! local rent control but still permits vacancy decontrol; (3) **NYC Rent
//! Stabilization Major Capital Improvement (MCI)** under 9 NYCRR § 2522.4
//! permits owner to pass through qualified capital improvement (boiler +
//! windows + electrical rewiring + plumbing + roofs) as permanent rent
//! increase amortized over 12 years (post-HSTPA 2019); HSTPA capped MCI
//! pass-through at 2% per year for rent-stabilized units; **30-year
//! removal** of MCI increase from rent calendar effective 2019; (4)
//! **NYC Individual Apartment Improvement (IAI)** under 9 NYCRR § 2522.4
//! permits unit-specific improvement pass-through capped at $15,000 over
//! 15 years (HSTPA cap); (5) **San Francisco Rent Ordinance § 37**
//! permits limited operating-cost pass-through after Rent Board petition;
//! (6) Boston rent control was REPEALED 1994 — common-law contract
//! freedom permits any lease-authorized pass-through.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    /// California Prop 13 + Costa-Hawkins + state AB 1482.
    California,
    /// NYC rent stabilization MCI + IAI post-HSTPA 2019.
    NewYorkCity,
    /// San Francisco Rent Ordinance § 37.
    SanFrancisco,
    /// Boston (rent control repealed 1994) — common-law contract.
    Boston,
    /// Default — state-specific rent control or common-law contract.
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitClass {
    /// Pre-1995 multifamily covered by local rent control.
    Pre1995MultifamilyRentControlled,
    /// Post-1995 multifamily Costa-Hawkins exempt + AB 1482 capped.
    Post1995CostaHawkinsExempt,
    /// Single-family or condo — Costa-Hawkins exempt.
    SingleFamilyOrCondo,
    /// NYC rent-stabilized unit.
    NycRentStabilized,
    /// NYC market-rate unit.
    NycMarketRate,
    /// Non-rent-controlled unit in any jurisdiction.
    NonRentControlled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PassThroughType {
    /// Routine property tax reassessment.
    PropertyTaxReassessment,
    /// Major Capital Improvement (boiler + windows + roof + plumbing).
    MajorCapitalImprovementMci,
    /// Individual Apartment Improvement (in-unit upgrades).
    IndividualApartmentImprovementIai,
    /// Special assessment district / local improvement district fee.
    SpecialAssessmentLid,
    /// Operating-cost increase petition.
    OperatingCostIncreasePetition,
    /// No pass-through attempted.
    NoPassThrough,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    CompliantPassThroughAuthorized,
    UnauthorizedPassThroughLeaseClauseMissing,
    MciExceeds2PctAnnualCapViolation,
    IaiExceeds15KOver15YearCapViolation,
    PropertyTaxPassThroughVoidNonCommercial,
    MciNotRegisteredWithRentBoardViolation,
    ChangeOfOwnershipReassessmentImproperlyPassed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub unit_class: UnitClass,
    pub pass_through_type: PassThroughType,
    pub lease_contains_pass_through_clause: bool,
    pub mci_or_iai_registered_with_rent_board: bool,
    pub annual_pass_through_amount_cents: u64,
    pub annual_rent_cents: u64,
    pub mci_iai_total_amortized_cents: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub recommended_actions: Vec<String>,
    pub annual_rent_at_risk_cents: u64,
    pub permitted_pass_through_amount_cents: u64,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub const NYC_MCI_HSTPA_ANNUAL_CAP_BPS: u32 = 200;
pub const NYC_MCI_AMORTIZATION_YEARS: u32 = 12;
pub const NYC_MCI_REMOVAL_YEARS_POST_HSTPA: u32 = 30;
pub const NYC_IAI_MAX_AMORTIZED_CENTS: u64 = 1_500_000;
pub const NYC_IAI_AMORTIZATION_YEARS: u32 = 15;
pub const CA_PROP_13_ANNUAL_CAP_BPS: u32 = 200;
pub const CA_PROP_13_ENACTED_YEAR: i32 = 1978;
pub const COSTA_HAWKINS_YEAR_THRESHOLD: i32 = 1995;
pub const NYC_HSTPA_YEAR: i32 = 2019;

pub fn check(input: &Input) -> Output {
    let mut actions: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    let severity: Severity;
    let mut permitted: u64 = 0;

    if matches!(input.pass_through_type, PassThroughType::NoPassThrough) {
        notes.push(
            "No pass-through attempted — framework inapplicable. Confirm lease language \
             remains compliant with jurisdiction-specific rent-control or rent-stabilization \
             requirements for future planning."
                .to_string(),
        );
        return Output {
            severity: Severity::NotApplicable,
            recommended_actions: actions,
            annual_rent_at_risk_cents: 0,
            permitted_pass_through_amount_cents: 0,
            citation: "n/a",
            notes,
        };
    }

    if matches!(
        input.pass_through_type,
        PassThroughType::PropertyTaxReassessment
    ) && matches!(input.jurisdiction, Jurisdiction::California)
        && !input.lease_contains_pass_through_clause
    {
        severity = Severity::PropertyTaxPassThroughVoidNonCommercial;
        actions.push(format!(
            "California Proposition 13 (enacted {}) reassessment-based property tax pass-\
             through to RESIDENTIAL tenant requires EXPLICIT lease clause; absent clause, \
             pass-through is VOID. Commercial leases routinely include Prop 13 pass-through \
             clauses (allocate change-of-ownership reassessment + new construction); \
             residential leases generally do not. Remove pass-through from rent invoices; \
             refund unauthorized collections plus interest under Cal. Civ. Code § 3287; tenant \
             may invoke statutory penalty under Cal. Civ. Code § 1947.13 for unauthorized \
             rent charges.",
            CA_PROP_13_ENACTED_YEAR
        ));
    } else if matches!(input.jurisdiction, Jurisdiction::NewYorkCity)
        && matches!(input.unit_class, UnitClass::NycRentStabilized)
        && matches!(input.pass_through_type, PassThroughType::MajorCapitalImprovementMci)
    {
        let annual_cap = (u128::from(input.annual_rent_cents)
            * u128::from(NYC_MCI_HSTPA_ANNUAL_CAP_BPS)
            / 10_000) as u64;
        permitted = annual_cap;
        if input.annual_pass_through_amount_cents > annual_cap {
            severity = Severity::MciExceeds2PctAnnualCapViolation;
            actions.push(format!(
                "NYC MCI pass-through of {} cents exceeds HSTPA 2019 annual cap of 2% of \
                 base rent ({} cents). Reduce to cap; refund overcharge plus interest per \
                 9 NYCRR § 2526.1. MCI amortized over {}-year period; pass-through expires \
                 {} years after effective date per HSTPA 2019 reform. File DHCR Form OA-1 \
                 amended schedule.",
                input.annual_pass_through_amount_cents,
                annual_cap,
                NYC_MCI_AMORTIZATION_YEARS,
                NYC_MCI_REMOVAL_YEARS_POST_HSTPA
            ));
        } else if !input.mci_or_iai_registered_with_rent_board {
            severity = Severity::MciNotRegisteredWithRentBoardViolation;
            actions.push(
                "MCI pass-through not registered with NYS Division of Housing and Community \
                 Renewal (DHCR); per 9 NYCRR § 2522.4 owner MUST file MCI rent increase \
                 application with DHCR before collecting; collections prior to DHCR approval \
                 are subject to refund plus treble damages plus attorney's fees under \
                 Emergency Tenant Protection Act § 11."
                    .to_string(),
            );
        } else {
            severity = Severity::CompliantPassThroughAuthorized;
            actions.push(format!(
                "Compliant MCI pass-through: amount within 2% annual cap, DHCR-registered, \
                 amortization period {} years; pass-through expires {} years after \
                 effective date per HSTPA 2019 reform.",
                NYC_MCI_AMORTIZATION_YEARS, NYC_MCI_REMOVAL_YEARS_POST_HSTPA
            ));
        }
    } else if matches!(input.jurisdiction, Jurisdiction::NewYorkCity)
        && matches!(input.unit_class, UnitClass::NycRentStabilized)
        && matches!(
            input.pass_through_type,
            PassThroughType::IndividualApartmentImprovementIai
        )
    {
        permitted = NYC_IAI_MAX_AMORTIZED_CENTS / u64::from(NYC_IAI_AMORTIZATION_YEARS);
        if input.mci_iai_total_amortized_cents > NYC_IAI_MAX_AMORTIZED_CENTS {
            severity = Severity::IaiExceeds15KOver15YearCapViolation;
            actions.push(format!(
                "NYC IAI total amortized amount of {} cents exceeds HSTPA 2019 ${} cap over \
                 {}-year amortization. Reduce to cap; refund overcharge per 9 NYCRR § 2522.4 \
                 plus DHCR Fact Sheet #26.",
                input.mci_iai_total_amortized_cents,
                NYC_IAI_MAX_AMORTIZED_CENTS / 100,
                NYC_IAI_AMORTIZATION_YEARS
            ));
        } else {
            severity = Severity::CompliantPassThroughAuthorized;
            actions.push(format!(
                "Compliant IAI pass-through: total amortized {} cents within ${} HSTPA cap; \
                 amortized over {} years; preserve receipts plus work-permit documentation \
                 for DHCR audit window.",
                input.mci_iai_total_amortized_cents,
                NYC_IAI_MAX_AMORTIZED_CENTS / 100,
                NYC_IAI_AMORTIZATION_YEARS
            ));
        }
    } else if matches!(input.jurisdiction, Jurisdiction::California)
        && matches!(input.pass_through_type, PassThroughType::PropertyTaxReassessment)
        && input.lease_contains_pass_through_clause
        && matches!(input.unit_class, UnitClass::Pre1995MultifamilyRentControlled)
    {
        severity = Severity::ChangeOfOwnershipReassessmentImproperlyPassed;
        actions.push(format!(
            "California pre-{} rent-controlled unit: Prop 13 change-of-ownership reassessment \
             cannot be passed through to RENT-CONTROLLED tenant even with lease clause; \
             local rent boards (LA RSO + SF Rent Ordinance + Oakland Rent Adjustment Program \
             + Berkeley Rent Stabilization Board) preempt landlord-tenant contract on \
             reassessment pass-through. Costa-Hawkins exempts post-{} construction + SFR / \
             condos.",
            COSTA_HAWKINS_YEAR_THRESHOLD, COSTA_HAWKINS_YEAR_THRESHOLD
        ));
    } else if !input.lease_contains_pass_through_clause {
        severity = Severity::UnauthorizedPassThroughLeaseClauseMissing;
        actions.push(
            "Pass-through attempted without explicit lease clause authorizing the charge. \
             Common-law contract requires landlord-tenant agreement on additional charges \
             beyond agreed base rent. Cease pass-through; refund unauthorized collections \
             with statutory interest; amend lease at next renewal to include explicit \
             pass-through provision if economically rational."
                .to_string(),
        );
    } else {
        severity = Severity::CompliantPassThroughAuthorized;
        actions.push(
            "Compliant pass-through: lease clause authorizes the charge, jurisdiction-\
             specific rent-control restrictions satisfied. Maintain documentation of \
             pass-through computation plus tenant invoice plus rent-board registration if \
             applicable for full statute-of-limitations window."
                .to_string(),
        );
    }

    match input.jurisdiction {
        Jurisdiction::California => {
            notes.push(format!(
                "California Proposition 13 (enacted {}) caps annual property tax increases \
                 at 2% UNTIL change-of-ownership or new construction triggers reassessment \
                 to market value (Cal. Const. Art. XIII A § 1). Costa-Hawkins Rental Housing \
                 Act (Cal. Civ. Code § 1954.50-.535) exempts post-{} construction + single-\
                 family / condominium units from local rent control. AB 1482 statewide rent \
                 cap (5% + CPI) applies to non-Costa-Hawkins-exempt units. Local rent boards \
                 preempt landlord-tenant contract on reassessment pass-through for rent-\
                 controlled units.",
                CA_PROP_13_ENACTED_YEAR, COSTA_HAWKINS_YEAR_THRESHOLD
            ));
        }
        Jurisdiction::NewYorkCity => {
            notes.push(format!(
                "NYC Rent Stabilization Major Capital Improvement (MCI) under 9 NYCRR § \
                 2522.4 permits owner pass-through of qualified capital improvement (boiler \
                 + windows + electrical rewiring + plumbing + roofs) amortized over {} \
                 years; Housing Stability and Tenant Protection Act of {} (HSTPA) capped \
                 MCI annual pass-through at 2% of base rent + reduced amortization + {} \
                 year removal from rent calendar. NYC Individual Apartment Improvement \
                 (IAI) capped at ${} total over {}-year amortization. NYS DHCR enforcement.",
                NYC_MCI_AMORTIZATION_YEARS,
                NYC_HSTPA_YEAR,
                NYC_MCI_REMOVAL_YEARS_POST_HSTPA,
                NYC_IAI_MAX_AMORTIZED_CENTS / 100,
                NYC_IAI_AMORTIZATION_YEARS
            ));
        }
        Jurisdiction::SanFrancisco => {
            notes.push(
                "SF Rent Ordinance § 37 (administered by SF Rent Board) permits limited \
                 operating-cost pass-through after Rent Board petition; capital improvement \
                 pass-through requires Rent Board approval per § 37.7. SF Admin Code \
                 prohibits unauthorized pass-through; § 37.10B impositions void."
                    .to_string(),
            );
        }
        Jurisdiction::Boston => {
            notes.push(
                "Boston rent control was REPEALED by Massachusetts ballot initiative in 1994 \
                 (Mass. Gen. Laws ch. 40P) — common-law contract freedom permits any lease-\
                 authorized pass-through. Mayor Wu rent-stabilization proposal pending state \
                 enabling legislation. M.G.L. ch. 186 § 14 quiet enjoyment may limit \
                 punitive pass-throughs."
                    .to_string(),
            );
        }
        Jurisdiction::Default => {
            notes.push(
                "Outside major rent-controlled jurisdictions, state-specific rent control \
                 (Oregon SB 608 statewide cap + Maine + Minneapolis ballot rent control + \
                 NJ municipal rent control) or common-law contract governs pass-through. \
                 Verify state landlord-tenant statute plus local rent-board jurisdiction \
                 before passing through any tax or improvement charge."
                    .to_string(),
            );
        }
    }

    notes.push(
        "Coordination with [[rental_property_registration]] (registration framework — \
         distinct compliance pathway), [[rental_hoa_disclosure_at_lease]] (HOA fees), \
         [[rental_solar_panel_disclosure]] (iter 495 PPA pass-through analog), [[rental_in_\
         unit_laundry_appliance_provision]] (iter 501 appliance maintenance distinct), \
         [[tenant_late_fee_cap]] (late-fee framework distinct charge), [[rental_junk_fee_\
         transparency]] (junk-fee disclosure framework), [[mid_tenancy_temporary_relocation]] \
         (when MCI work displaces tenant), [[rental_balcony_inspection_seismic_safety]] \
         (iter 511 — EEE inspection cost may qualify as MCI for NYC rent-stabilized units)."
            .to_string(),
    );

    let annual_rent_at_risk: u64 = match severity {
        Severity::PropertyTaxPassThroughVoidNonCommercial
        | Severity::ChangeOfOwnershipReassessmentImproperlyPassed
        | Severity::MciNotRegisteredWithRentBoardViolation => input.annual_rent_cents,
        Severity::MciExceeds2PctAnnualCapViolation
        | Severity::IaiExceeds15KOver15YearCapViolation
        | Severity::UnauthorizedPassThroughLeaseClauseMissing => {
            input.annual_rent_cents.saturating_div(2)
        }
        _ => 0,
    };

    Output {
        severity,
        recommended_actions: actions,
        annual_rent_at_risk_cents: annual_rent_at_risk,
        permitted_pass_through_amount_cents: permitted,
        citation: match input.jurisdiction {
            Jurisdiction::California => {
                "Cal. Const. Art. XIII A § 1 + Cal. Civ. Code § 1954.50-.535 + AB 1482"
            }
            Jurisdiction::NewYorkCity => "9 NYCRR § 2522.4 + § 2526.1 + HSTPA 2019",
            Jurisdiction::SanFrancisco => "SF Rent Ordinance § 37 + § 37.7 + § 37.10B",
            Jurisdiction::Boston => "Mass. Gen. Laws ch. 40P (rent control repealed 1994)",
            Jurisdiction::Default => "State landlord-tenant + local rent-board + common-law",
        },
        notes,
    }
}

pub type RentalPropertyTaxPassThroughDisclosureInput = Input;
pub type RentalPropertyTaxPassThroughDisclosureResult = Output;

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Input {
        Input {
            jurisdiction: Jurisdiction::NewYorkCity,
            unit_class: UnitClass::NycRentStabilized,
            pass_through_type: PassThroughType::MajorCapitalImprovementMci,
            lease_contains_pass_through_clause: true,
            mci_or_iai_registered_with_rent_board: true,
            annual_pass_through_amount_cents: 480_00,
            annual_rent_cents: 36_000_00,
            mci_iai_total_amortized_cents: 10_000_00,
        }
    }

    #[test]
    fn no_pass_through_not_applicable() {
        let mut i = baseline();
        i.pass_through_type = PassThroughType::NoPassThrough;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NotApplicable));
        assert_eq!(r.annual_rent_at_risk_cents, 0);
    }

    #[test]
    fn compliant_mci_within_2_pct_cap() {
        let i = baseline();
        let r = check(&i);
        assert!(matches!(r.severity, Severity::CompliantPassThroughAuthorized));
        assert_eq!(r.permitted_pass_through_amount_cents, 720_00);
        assert_eq!(r.annual_rent_at_risk_cents, 0);
    }

    #[test]
    fn mci_exceeds_2_pct_cap_violation() {
        let mut i = baseline();
        i.annual_pass_through_amount_cents = 1_200_00;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::MciExceeds2PctAnnualCapViolation));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents / 2);
        assert!(r.recommended_actions.iter().any(|a| a.contains("HSTPA 2019")));
    }

    #[test]
    fn mci_not_registered_violation() {
        let mut i = baseline();
        i.mci_or_iai_registered_with_rent_board = false;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::MciNotRegisteredWithRentBoardViolation
        ));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents);
        assert!(r.recommended_actions.iter().any(|a| a.contains("DHCR")));
        assert!(r.recommended_actions.iter().any(|a| a.contains("treble damages")));
    }

    #[test]
    fn ca_property_tax_pass_through_without_lease_clause_void() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::California;
        i.unit_class = UnitClass::Post1995CostaHawkinsExempt;
        i.pass_through_type = PassThroughType::PropertyTaxReassessment;
        i.lease_contains_pass_through_clause = false;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::PropertyTaxPassThroughVoidNonCommercial
        ));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents);
        assert!(r.recommended_actions.iter().any(|a| a.contains("Proposition 13")));
        assert!(r.recommended_actions.iter().any(|a| a.contains("§ 1947.13")));
    }

    #[test]
    fn ca_pre_1995_rent_controlled_change_of_ownership_improper() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::California;
        i.unit_class = UnitClass::Pre1995MultifamilyRentControlled;
        i.pass_through_type = PassThroughType::PropertyTaxReassessment;
        i.lease_contains_pass_through_clause = true;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::ChangeOfOwnershipReassessmentImproperlyPassed
        ));
        assert!(r.recommended_actions.iter().any(|a| a.contains("Costa-Hawkins")));
        assert!(r.recommended_actions.iter().any(|a| a.contains("Berkeley")));
    }

    #[test]
    fn iai_exceeds_15k_cap_violation() {
        let mut i = baseline();
        i.pass_through_type = PassThroughType::IndividualApartmentImprovementIai;
        i.mci_iai_total_amortized_cents = 20_000_00;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::IaiExceeds15KOver15YearCapViolation));
        assert!(r.recommended_actions.iter().any(|a| a.contains("15000")));
    }

    #[test]
    fn iai_within_15k_cap_compliant() {
        let mut i = baseline();
        i.pass_through_type = PassThroughType::IndividualApartmentImprovementIai;
        i.mci_iai_total_amortized_cents = 10_000_00;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::CompliantPassThroughAuthorized));
    }

    #[test]
    fn no_lease_clause_unauthorized_pass_through() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Default;
        i.unit_class = UnitClass::NonRentControlled;
        i.pass_through_type = PassThroughType::SpecialAssessmentLid;
        i.lease_contains_pass_through_clause = false;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::UnauthorizedPassThroughLeaseClauseMissing
        ));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents / 2);
    }

    #[test]
    fn ca_jurisdiction_pins_prop_13_and_costa_hawkins() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::California;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("Proposition 13")));
        assert!(r.notes.iter().any(|n| n.contains("Costa-Hawkins")));
        assert!(r.notes.iter().any(|n| n.contains("AB 1482")));
        assert!(r.notes.iter().any(|n| n.contains("1978")));
        assert!(r.notes.iter().any(|n| n.contains("1995")));
    }

    #[test]
    fn nyc_jurisdiction_pins_hstpa_2019_and_dhcr() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("HSTPA")));
        assert!(r.notes.iter().any(|n| n.contains("2019")));
        assert!(r.notes.iter().any(|n| n.contains("DHCR")));
        assert!(r.notes.iter().any(|n| n.contains("9 NYCRR § 2522.4")));
        assert!(r.notes.iter().any(|n| n.contains("30")));
    }

    #[test]
    fn sf_jurisdiction_pins_rent_ordinance_37() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::SanFrancisco;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("SF Rent Ordinance § 37")));
        assert!(r.notes.iter().any(|n| n.contains("§ 37.7")));
    }

    #[test]
    fn boston_jurisdiction_pins_rent_control_repealed_1994() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Boston;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("ch. 40P")));
        assert!(r.notes.iter().any(|n| n.contains("REPEALED")));
        assert!(r.notes.iter().any(|n| n.contains("1994")));
        assert!(r.notes.iter().any(|n| n.contains("Mayor Wu")));
    }

    #[test]
    fn default_jurisdiction_pins_state_specific() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Default;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("Oregon SB 608")));
        assert!(r.notes.iter().any(|n| n.contains("Minneapolis")));
    }

    #[test]
    fn coordination_note_references_siblings() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("rental_property_registration")));
        assert!(r.notes.iter().any(|n| n.contains("rental_hoa_disclosure_at_lease")));
        assert!(r.notes.iter().any(|n| n.contains("tenant_late_fee_cap")));
        assert!(r.notes.iter().any(|n| n.contains("rental_junk_fee_transparency")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("rental_balcony_inspection_seismic_safety")));
    }

    #[test]
    fn nyc_mci_hstpa_annual_cap_pins_2_pct() {
        assert_eq!(NYC_MCI_HSTPA_ANNUAL_CAP_BPS, 200);
    }

    #[test]
    fn nyc_mci_amortization_pins_12_years() {
        assert_eq!(NYC_MCI_AMORTIZATION_YEARS, 12);
    }

    #[test]
    fn nyc_mci_removal_post_hstpa_pins_30_years() {
        assert_eq!(NYC_MCI_REMOVAL_YEARS_POST_HSTPA, 30);
    }

    #[test]
    fn nyc_iai_max_amortized_pins_15000_dollars() {
        assert_eq!(NYC_IAI_MAX_AMORTIZED_CENTS, 1_500_000);
    }

    #[test]
    fn nyc_iai_amortization_pins_15_years() {
        assert_eq!(NYC_IAI_AMORTIZATION_YEARS, 15);
    }

    #[test]
    fn ca_prop_13_annual_cap_pins_2_pct() {
        assert_eq!(CA_PROP_13_ANNUAL_CAP_BPS, 200);
    }

    #[test]
    fn ca_prop_13_enacted_year_pins_1978() {
        assert_eq!(CA_PROP_13_ENACTED_YEAR, 1978);
    }

    #[test]
    fn costa_hawkins_year_pins_1995() {
        assert_eq!(COSTA_HAWKINS_YEAR_THRESHOLD, 1995);
    }

    #[test]
    fn nyc_hstpa_year_pins_2019() {
        assert_eq!(NYC_HSTPA_YEAR, 2019);
    }

    #[test]
    fn citation_branch_for_each_jurisdiction() {
        let ca = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::California; i });
        let nyc = check(&baseline());
        let sf = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::SanFrancisco; i });
        let bos = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::Boston; i });
        let de = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::Default; i });
        assert!(ca.citation.contains("Cal. Const."));
        assert!(nyc.citation.contains("9 NYCRR"));
        assert!(sf.citation.contains("SF Rent Ordinance"));
        assert!(bos.citation.contains("ch. 40P"));
        assert!(de.citation.contains("State landlord-tenant"));
    }

    #[test]
    fn zero_rent_zero_risk_does_not_panic() {
        let mut i = baseline();
        i.annual_rent_cents = 0;
        i.lease_contains_pass_through_clause = false;
        let r = check(&i);
        assert_eq!(r.annual_rent_at_risk_cents, 0);
    }
}
